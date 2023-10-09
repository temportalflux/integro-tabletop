pub static USER_HOMEBREW_REPO_NAME: &str = "integro-homebrew";

pub mod autosync;
pub mod github;

#[yew::hook]
pub fn use_storage() -> Option<github::GithubClient> {
	let client = yew::use_state(|| None::<github::GithubClient>);
	let auth_status = yewdux::prelude::use_store_value::<crate::auth::Status>();
	yew::use_effect_with_deps(
		{
			let client = client.clone();
			move |auth_status: &std::rc::Rc<crate::auth::Status>| {
				client.set(match auth_status.as_ref() {
					crate::auth::Status::Successful { provider, token } => match provider {
						crate::auth::OAuthProvider::Github => match github::GithubClient::new(token) {
							Ok(storage) => Some(storage),
							Err(err) => {
								log::error!(target: "storage", "{err:?}");
								None
							}
						},
					},
					_ => None,
				})
			}
		},
		auth_status,
	);
	(*client).clone()
}
