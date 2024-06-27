use crate::{auth, storage::autosync};
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Store, Default)]
#[store(storage = "session", storage_tab_sync)]
pub struct LocalUser(Option<crate::data::UserId>);

impl From<crate::data::UserId> for LocalUser {
	fn from(value: crate::data::UserId) -> Self {
		Self(Some(value))
	}
}

impl LocalUser {
	pub fn homebrew_module_id(&self) -> Option<crate::system::ModuleId> {
		let Some(user_id) = &self.0 else { return None };
		Some(crate::system::ModuleId::Github {
			user_org: user_id.id.clone(),
			repository: crate::storage::USER_HOMEBREW_REPO_NAME.to_owned(),
		})
	}
}

#[hook]
pub fn use_on_auth_success<F>(callback: F)
where
	F: Fn(&std::rc::Rc<auth::Status>) + 'static,
{
	let callback = yew_hooks::use_latest(callback);
	let auth_status = use_store_value::<auth::Status>();
	let was_success = use_state_eq({
		let auth_status = auth_status.clone();
		move || matches!(*auth_status, auth::Status::Successful { .. })
	});
	use_effect_with((auth_status, was_success), move |(status, was_authenticated)| {
		let is_authenticated = matches!(**status, auth::Status::Successful { .. });
		if is_authenticated && !**was_authenticated {
			(*callback.current())(status);
		}
		was_authenticated.set(is_authenticated);
	});
}

#[function_component]
pub fn LoginButton() -> Html {
	let auth = use_context::<auth::Auth>().unwrap();
	let auth_status = use_store_value::<auth::Status>();
	let autosync_status = use_context::<autosync::Status>().unwrap();
	let disabled = autosync_status.is_active();
	if matches!(*auth_status, auth::Status::Successful { .. }) {
		let onclick = auth.logout_callback().clone();
		let onclick = Callback::from(move |_| {
			onclick.emit(());
			yewdux::Dispatch::<LocalUser>::global().set(LocalUser::default());
		});
		html! {
			<button
				class="btn btn-outline-danger"
				{disabled}
				{onclick}
			>
				{"Sign Out"}
			</button>
		}
	} else {
		let onclick = auth.login_callback().reform(|_: MouseEvent| auth::OAuthProvider::Github.request());
		html! {
			<button
				class="btn btn-success"
				{disabled}
				{onclick}
			>
				{"Sign In"}
			</button>
		}
	}
}
