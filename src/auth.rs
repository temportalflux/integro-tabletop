use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::sync::Mutex;
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
		token: String,
	},
	Failed {
		error: String,
	},
}

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

#[function_component]
pub fn ActionProvider(ChildrenProps { children }: &ChildrenProps) -> Html {
	let (auth_status, dispatch) = use_store::<Status>();
	let pending_auth_state = use_memo(|_| Mutex::new(None::<PendingAuthState>), ());
	let on_window_message = use_memo(
		|_| {
			let state = pending_auth_state.clone();
			Closure::<dyn Fn(_)>::new(move |evt: MessageEvent| {
				let Ok(mut state) = state.lock() else { return; };
				if let Some(pending) = state.take() {
					*state = pending.handle_event(evt);
				}
			})
		},
		(),
	);
	let logout = Callback::from({
		let dispatch = dispatch.clone();
		let state = pending_auth_state.clone();
		move |_: ()| {
			dispatch.set(Status::None);
			let Ok(mut state) = state.lock() else { return; };
			let _ = state.take();
		}
	});
	let reset_auth_state = use_memo(
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
		(),
	);
	let login = Callback::from(move |provider| match &*auth_status {
		Status::Successful { token: _ } => {}
		Status::Authorizing => {}
		Status::None | Status::Failed { error: _ } => {
			if let Some(auth_state) = PendingAuthState::authenticate(
				provider,
				&on_window_message,
				&reset_auth_state,
				&dispatch,
			) {
				let Ok(mut state) = pending_auth_state.lock() else { return; };
				*state = Some(auth_state);
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
		let scope = "repo,read:org,read:user,workflow";
		let base_url = "https://api.netlify.com";
		let auth_url =
			format!("{base_url}/auth?provider={provider_id}&site_id={SITE_ID}&scope={scope}");
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
		let Ok(Some(auth_window)) = window.open_with_url_and_target_and_features(&auth_url, "Tabletop Tools Authorization", &features) else { return None; };

		let timeout_ms = 1000 * 60;
		let _ = window.remove_event_listener_with_callback(
			"message",
			(**on_window_message).as_ref().unchecked_ref(),
		);
		let _ = window.add_event_listener_with_callback(
			"message",
			(**on_window_message).as_ref().unchecked_ref(),
		);
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
		let Ok(data_str) = evt.data().into_serde::<String>() else { return None; };

		let prefix = match self.handshake_established {
			false => "authorizing",
			true => "authorization",
		};
		let auth_header = format!("{prefix}:{}", self.provider.oauth_id());
		if !data_str.starts_with(&auth_header) || evt.origin() != self.base_url {
			return None;
		}
		if !self.handshake_established {
			self.handshake_established = true;
			let _ = self.auth_window.post_message(&evt.data(), &evt.origin());
			return Some(self);
		}

		let _ = self.auth_window.close();
		let Some(data_state) = data_str.strip_prefix(&format!("{auth_header}:")) else { return None; };
		if let Some(success_data) = data_state.strip_prefix("success:") {
			let Ok(data) = serde_json::from_str::<serde_json::Value>(success_data) else {return None;};
			let Some(auth_token) = data.get("token") else { return None; };
			let Some(token) = auth_token.as_str() else { return None; };
			self.auth_status.set(Status::Successful {
				token: token.to_owned(),
			});
		} else if let Some(error_data) = data_state.strip_prefix("error:") {
			let Ok(data) = serde_json::from_str::<serde_json::Value>(error_data) else {return None;};
			log::debug!("error: {data:?}");
			self.auth_status.set(Status::Failed { error: "".into() });
		} else {
			self.auth_status.set(Status::None);
		}
		None
	}
}
