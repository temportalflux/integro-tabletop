use crate::auth;
use yew::prelude::*;
use yewdux::prelude::*;

#[hook]
pub fn use_on_auth_success<F>(callback: F) where F: Fn(&std::rc::Rc<auth::Status>) + 'static {
	let callback = yew_hooks::use_latest(callback);
	let auth_status = use_store_value::<auth::Status>();
	let was_success = use_state_eq({
		let auth_status = auth_status.clone();
		move || matches!(*auth_status, auth::Status::Successful { .. })
	});
	use_effect_with_deps(move |(status, was_authenticated)| {
		let is_authenticated = matches!(**status, auth::Status::Successful { .. });
		if is_authenticated && !**was_authenticated {
			(*callback.current())(status);
		}
		was_authenticated.set(is_authenticated);
	}, (auth_status, was_success));
}

#[function_component]
pub fn LoginButton() -> Html {
	let (auth_status, _) = use_store::<auth::Status>();
	let auth = use_context::<auth::Auth>().unwrap();
	if matches!(*auth_status, auth::Status::Successful { .. }) {
		let onclick = auth.logout_callback().reform(|_: MouseEvent| ());
		html! {
			<button
				class="btn btn-outline-danger"
				{onclick}
			>
				{"Sign Out"}
			</button>
		}
	} else {
		let onclick = auth
			.login_callback()
			.reform(|_: MouseEvent| auth::OAuthProvider::Github);
		html! {
			<button
				class="btn btn-success"
				{onclick}
			>
				{"Sign In"}
			</button>
		}
	}
}
