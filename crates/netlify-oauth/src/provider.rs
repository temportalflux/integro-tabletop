use crate::{Auth, OAuthId, Request, Status};
use gloo_utils::format::JsValueSerdeExt;
use std::{collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::MessageEvent;
use yew::{html::ChildrenProps, prelude::*};
use yewdux::prelude::*;

// Based on: https://www.npmjs.com/package/netlify-auth-providers?activeTab=code
struct Pending {
	base_url: &'static str,
	provider_id: OAuthId,
	auth_window: web_sys::Window,
	handshake_established: bool,
}

enum AuthMessage {
	Reset,
	Initiated(Pending),
	Event(MessageEvent),
	Clear,
}

#[function_component]
pub fn Provider(ChildrenProps { children }: &ChildrenProps) -> Html {
	let (auth_status, dispatch) = use_store::<Status>();
	let message_channel = use_state(|| async_channel::unbounded::<AuthMessage>());
	yew_hooks::use_async_with_options(
		{
			let recv_msg = message_channel.1.clone();
			async move {
				let mut state = None::<Pending>;
				let mut backlog = VecDeque::new();
				while let Ok(event) = recv_msg.recv().await {
					match event {
						AuthMessage::Reset => {
							backlog.clear();
						}
						AuthMessage::Initiated(new_state) => {
							state = Some(new_state);
							dispatch.set(Status::Authorizing);
						}
						AuthMessage::Event(evt) => {
							backlog.push_back(evt);
						}
						AuthMessage::Clear => {
							state = None;
							dispatch.set(Status::None);
						}
					}
					if state.is_some() && !backlog.is_empty() {
						let mut tabled = VecDeque::new();
						for evt in backlog.drain(..) {
							match state.take() {
								None => tabled.push_back(evt),
								Some(old_state) => match old_state.handle_event(evt) {
									EventResponse::Ignored(pending) => {
										state = Some(pending);
									}
									EventResponse::Finished(status) => {
										dispatch.set(status);
									}
								},
							}
						}
						backlog = tabled;
					}
				}
				Ok(()) as Result<(), ()>
			}
		},
		yew_hooks::UseAsyncOptions::enable_auto(),
	);

	let on_window_message = use_memo((), |_| {
		let send_msg = message_channel.0.clone();
		Closure::<dyn Fn(_)>::new(move |evt: MessageEvent| {
			let _ = send_msg.try_send(AuthMessage::Event(evt));
		})
	});
	let logout = Callback::from({
		let send_msg = message_channel.0.clone();
		move |_: ()| {
			let _ = send_msg.try_send(AuthMessage::Clear);
		}
	});
	let on_timeout = use_memo((), |_| {
		let auth_status = auth_status.clone();
		let logout = logout.clone();
		Closure::<dyn Fn()>::new(move || {
			if *auth_status == Status::Authorizing {
				log::debug!("Authorizing took too long, resetting auth status.");
				logout.emit(());
			}
		})
	});
	let login = Callback::from({
		let send_msg = message_channel.0.clone();
		move |request: Request| match &*auth_status {
			Status::Successful { .. } => {}
			Status::Authorizing | Status::None | Status::Failed { error: _ } => {
				let _ = send_msg.try_send(AuthMessage::Reset);
				let pending = Pending::initiate(request, &on_window_message, &on_timeout);
				if let Some(pending) = pending {
					let _ = send_msg.try_send(AuthMessage::Initiated(pending));
				}
			}
		}
	});

	let auth = Auth { login, logout };
	html! {
		<ContextProvider<Auth> context={auth.clone()}>
			{children.clone()}
		</ContextProvider<Auth>>
	}
}

enum EventResponse {
	Ignored(Pending),
	Finished(Status),
}

impl Pending {
	fn initiate(
		request: Request,
		on_window_message: &Rc<Closure<dyn Fn(MessageEvent)>>,
		on_timeout: &Rc<Closure<dyn Fn()>>,
	) -> Option<Self> {
		let scope = "repo,read:org,read:user";
		let base_url = "https://api.netlify.com";
		let Request {
			provider_id,
			site_id,
			window_title,
		} = request;
		let auth_url = format!("{base_url}/auth?provider={provider_id}&site_id={site_id}&scope={scope}");
		let Some(window) = web_sys::window() else {
			return None;
		};
		let Ok(screen) = window.screen() else {
			return None;
		};
		let width = 960;
		let height = 960;
		let top = screen.width().unwrap_or(0) / 2 - width / 2;
		let left = screen.height().unwrap_or(0) / 2 - height / 2;
		let const_features = [
			"toolbar=no",
			"location=no",
			"directories=no",
			"status=no",
			"menubar=no",
			"scrollbars=no",
			"copyhistory=no",
		]
		.join(", ");
		let dyn_features = [
			format!("width={width}"),
			format!("height={height}"),
			format!("top={top}"),
			format!("left={left}"),
		]
		.join(", ");
		let features = format!("{const_features}, {dyn_features}");
		log::debug!(target: "auth", "Initializing login");
		let Ok(Some(auth_window)) =
			window.open_with_url_and_target_and_features(&auth_url, &window_title, &features)
		else {
			return None;
		};

		let timeout_ms = 1000 * 60;
		let _ = window.remove_event_listener_with_callback("message", (**on_window_message).as_ref().unchecked_ref());
		let _ = window.add_event_listener_with_callback("message", (**on_window_message).as_ref().unchecked_ref());
		let _ = window
			.set_timeout_with_callback_and_timeout_and_arguments_0((**on_timeout).as_ref().unchecked_ref(), timeout_ms);
		let _ = auth_window.focus();
		Some(Self {
			base_url,
			provider_id,
			auth_window,
			handshake_established: false,
		})
	}

	fn handle_event(mut self, evt: MessageEvent) -> EventResponse {
		let Ok(data_str) = evt.data().into_serde::<String>() else {
				return EventResponse::Ignored(self);
			};

		let prefix = match self.handshake_established {
			false => "authorizing",
			true => "authorization",
		};
		let auth_header = format!("{prefix}:{}", self.provider_id);
		if !data_str.starts_with(&auth_header) || evt.origin() != self.base_url {
			log::error!(target: "auth", "Failed to find auth header: {auth_header:?}?={data_str:?} {:?}?={:?}", evt.origin(), self.base_url);
			return EventResponse::Ignored(self);
		}
		if !self.handshake_established {
			log::debug!(target: "auth", "Handshake established");
			self.handshake_established = true;
			let _ = self.auth_window.post_message(&evt.data(), &evt.origin());
			return EventResponse::Ignored(self);
		}

		log::debug!(target: "auth", "Closing auth window");

		let _ = self.auth_window.close();
		let Some(data_state) = data_str.strip_prefix(&format!("{auth_header}:")) else {
				log::error!(target: "auth", "Invalid data header {auth_header:?}?={data_str:?}");
				return EventResponse::Ignored(self);
			};
		let status = if let Some(success_data) = data_state.strip_prefix("success:") {
			let Ok(data) = serde_json::from_str::<serde_json::Value>(success_data) else {
					log::error!(target: "auth", "Failed to deserialize success from {success_data:?}");
					return EventResponse::Ignored(self);
				};
			let Some(auth_token) = data.get("token") else {
					log::error!(target: "auth", "Failed to deserialize auth token from {data:?}");
					return EventResponse::Ignored(self);
				};
			let Some(token) = auth_token.as_str() else {
					log::error!(target: "auth", "Failed to parse auth token from {auth_token:?}");
					return EventResponse::Ignored(self);
				};
			Status::Successful {
				oauth_id: self.provider_id.to_owned(),
				token: token.to_owned(),
			}
		} else if let Some(error_data) = data_state.strip_prefix("error:") {
			let Ok(data) = serde_json::from_str::<serde_json::Value>(error_data) else {
					log::error!(target: "auth", "Failed to deserialize error from {error_data:?}");
					return EventResponse::Ignored(self);
				};
			log::error!(target: "auth", "Failed authentication: {data:?}");
			Status::Failed { error: "".into() }
		} else {
			Status::None
		};
		EventResponse::Finished(status)
	}
}
