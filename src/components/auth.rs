use crate::auth;
use yew::prelude::*;
use yewdux::prelude::*;

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
