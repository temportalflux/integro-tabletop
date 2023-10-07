use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::MessageEvent;
use yew::{html::ChildrenProps, prelude::*};
use yewdux::prelude::*;

static SITE_ID: &str = "f48e4964-d583-424b-bace-bd51a12f72a2";

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
pub enum Status {
	#[default]
	None,
	Authorizing,
	Successful {
		provider: OAuthProvider,
		token: String,
	},
	Failed {
		error: String,
	},
}
impl Status {
	pub fn storage(&self) -> Option<crate::storage::github::GithubClient> {
		match self {
			Status::Successful {
				provider: OAuthProvider::Github,
				token,
			} => crate::storage::github::GithubClient::new(token).ok(),
			_ => None,
		}
	}
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum OAuthProvider {
	Github,
}
impl OAuthProvider {
	pub fn oauth_id(&self) -> &'static str {
		match self {
			Self::Github => "github",
		}
	}
}

enum AuthMessage {
	State(PendingAuthState),
	Event(MessageEvent),
	ClearPending,
	Reset,
}

#[function_component]
pub fn ActionProvider(ChildrenProps { children }: &ChildrenProps) -> Html {
	let (auth_status, dispatch) = use_store::<Status>();
	let message_channel = use_state(|| async_channel::unbounded::<AuthMessage>());
	yew_hooks::use_async_with_options(
		{
			let recv_msg = message_channel.1.clone();
			async move {
				let mut state = None::<PendingAuthState>;
				let mut backlog = VecDeque::new();
				while let Ok(event) = recv_msg.recv().await {
					match event {
						AuthMessage::Reset => {
							backlog.clear();
						}
						AuthMessage::Event(evt) => {
							backlog.push_back(evt);
						}
						AuthMessage::State(new_state) => {
							state = Some(new_state);
						}
						AuthMessage::ClearPending => {
							state = None;
						}
					}
					if state.is_some() && !backlog.is_empty() {
						let mut tabled = VecDeque::new();
						for evt in backlog.drain(..) {
							match state.take() {
								None => tabled.push_back(evt),
								Some(old_state) => {
									state = old_state.handle_event(evt);
								}
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
	let on_window_message = use_memo(
		(),
		|_| {
			let send_msg = message_channel.0.clone();
			Closure::<dyn Fn(_)>::new(move |evt: MessageEvent| {
				let _ = send_msg.try_send(AuthMessage::Event(evt));
			})
		},
	);
	let logout = Callback::from({
		let dispatch = dispatch.clone();
		let send_msg = message_channel.0.clone();
		move |_: ()| {
			dispatch.set(Status::None);
			let _ = send_msg.try_send(AuthMessage::ClearPending);
		}
	});
	let reset_auth_state = use_memo(
		(),
		|_| {
			let auth_status = auth_status.clone();
			let logout = logout.clone();
			Closure::<dyn Fn()>::new(move || {
				if *auth_status == Status::Authorizing {
					log::debug!("Authorizing took too long, resetting auth status.");
					logout.emit(());
				}
			})
		},
	);
	let login = Callback::from({
		let send_msg = message_channel.0.clone();
		move |provider| match &*auth_status {
			Status::Successful { .. } => {}
			Status::Authorizing | Status::None | Status::Failed { error: _ } => {
				let _ = send_msg.try_send(AuthMessage::Reset);
				if let Some(auth_state) =
					PendingAuthState::authenticate(provider, &on_window_message, &reset_auth_state, &dispatch)
				{
					let _ = send_msg.try_send(AuthMessage::State(auth_state));
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

#[derive(Clone, PartialEq)]
pub struct Auth {
	login: Callback<OAuthProvider>,
	logout: Callback<()>,
}
impl Auth {
	pub fn login_callback(&self) -> &Callback<OAuthProvider> {
		&self.login
	}

	pub fn logout_callback(&self) -> &Callback<()> {
		&self.logout
	}

	pub fn sign_in(&self, provider: OAuthProvider) {
		self.login.emit(provider);
	}

	pub fn sign_out(&self) {
		self.logout.emit(());
	}
}

// Based on: https://www.npmjs.com/package/netlify-auth-providers?activeTab=code
struct PendingAuthState {
	base_url: &'static str,
	provider: OAuthProvider,
	auth_window: web_sys::Window,
	handshake_established: bool,
	auth_status: Dispatch<Status>,
}

impl PendingAuthState {
	fn authenticate(
		provider: OAuthProvider,
		on_window_message: &Rc<Closure<dyn Fn(MessageEvent)>>,
		reset_auth_state: &Rc<Closure<dyn Fn()>>,
		auth_status: &Dispatch<Status>,
	) -> Option<Self> {
		let provider_id = provider.oauth_id();
		let scope = "repo,read:org,read:user";
		let base_url = "https://api.netlify.com";
		let auth_url = format!("{base_url}/auth?provider={provider_id}&site_id={SITE_ID}&scope={scope}");
		let Some(window) = web_sys::window() else { return None; };
		let Ok(screen) = window.screen() else { return None; };
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
		let Ok(Some(auth_window)) = window.open_with_url_and_target_and_features(&auth_url, "Integro Authorization", &features) else { return None; };

		let timeout_ms = 1000 * 60;
		let _ = window.remove_event_listener_with_callback("message", (**on_window_message).as_ref().unchecked_ref());
		let _ = window.add_event_listener_with_callback("message", (**on_window_message).as_ref().unchecked_ref());
		let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
			(**reset_auth_state).as_ref().unchecked_ref(),
			timeout_ms,
		);
		let _ = auth_window.focus();
		auth_status.set(Status::Authorizing);
		Some(Self {
			base_url,
			provider,
			auth_window,
			auth_status: auth_status.clone(),
			handshake_established: false,
		})
	}

	fn handle_event(mut self, evt: MessageEvent) -> Option<Self> {
		let Ok(data_str) = evt.data().into_serde::<String>() else {
			return Some(self);
		};

		let prefix = match self.handshake_established {
			false => "authorizing",
			true => "authorization",
		};
		let auth_header = format!("{prefix}:{}", self.provider.oauth_id());
		if !data_str.starts_with(&auth_header) || evt.origin() != self.base_url {
			log::error!(target: "auth", "Failed to find auth header: {auth_header:?}?={data_str:?} {:?}?={:?}", evt.origin(), self.base_url);
			return Some(self);
		}
		if !self.handshake_established {
			log::debug!(target: "auth", "Handshake established");
			self.handshake_established = true;
			let _ = self.auth_window.post_message(&evt.data(), &evt.origin());
			return Some(self);
		}

		log::debug!(target: "auth", "Closing auth window");

		let _ = self.auth_window.close();
		let Some(data_state) = data_str.strip_prefix(&format!("{auth_header}:")) else {
			log::error!(target: "auth", "Invalid data header {auth_header:?}?={data_str:?}");
			return Some(self);
		};
		let status = if let Some(success_data) = data_state.strip_prefix("success:") {
			let Ok(data) = serde_json::from_str::<serde_json::Value>(success_data) else {
				log::error!(target: "auth", "Failed to deserialize success from {success_data:?}");
				return Some(self);
			};
			let Some(auth_token) = data.get("token") else {
				log::error!(target: "auth", "Failed to deserialize auth token from {data:?}");
				return Some(self);
			};
			let Some(token) = auth_token.as_str() else {
				log::error!(target: "auth", "Failed to parse auth token from {auth_token:?}");
				return Some(self);
			};
			Status::Successful {
				provider: self.provider,
				token: token.to_owned(),
			}
		} else if let Some(error_data) = data_state.strip_prefix("error:") {
			let Ok(data) = serde_json::from_str::<serde_json::Value>(error_data) else {
				log::error!(target: "auth", "Failed to deserialize error from {error_data:?}");
				return Some(self);
			};
			log::debug!("error: {data:?}");
			Status::Failed { error: "".into() }
		} else {
			Status::None
		};
		log::debug!(target: "auth", "Updating auth status to {status:?}");
		self.auth_status.set(status);
		None
	}
}
