use crate::{
	auth,
	storage::github::{GithubClient, QueryError, RepositoryMetadata},
};
use std::sync::Arc;
use yew::prelude::*;
use yew_hooks::use_async;
use yewdux::prelude::*;

/// Page which displays the modules the user currently logged in has contributor access to.
#[function_component]
pub fn OwnedModules() -> Html {
	let (state, _) = use_store::<auth::Status>();
	let was_authed = use_state_eq(|| false);
	let fetch_github = use_async({
		let auth = state.clone();
		async move {
			use futures_util::StreamExt;
			let auth::Status::Successful { token } = &*auth else { return Ok(Vec::new()); };
			log::debug!("detected login {token:?}");
			let client = GithubClient::new(token)?;

			let user = client.viewer().await?.viewer.login;
			log::debug!("Found viewier {user:?}");

			let mut owners = vec![user];
			let mut find_all_orgs = client.find_all_orgs();
			while let Some(org_list) = find_all_orgs.next().await {
				owners.extend(org_list);
			}
			log::debug!("{owners:?}");

			let mut relevant_repos = Vec::new();
			for owner in &owners {
				log::debug!("searching {owner:?}");
				let mut stream = client.search_for_repos(owner);
				while let Some(repos) = stream.next().await {
					relevant_repos.extend(repos);
				}
			}
			log::debug!("Valid Repositories: {relevant_repos:?}");

			Ok(relevant_repos) as Result<Vec<RepositoryMetadata>, Arc<QueryError>>
		}
	});
	let signed_in = matches!(*state, auth::Status::Successful { .. });
	if signed_in != *was_authed {
		if *was_authed {
			was_authed.set(false);
		} else {
			was_authed.set(true);
			fetch_github.run();
		}
	}
	let content = match (signed_in, fetch_github.loading, &fetch_github.data) {
		(false, _, _) => html!("Not signed in"),
		(true, true, _) => html! {
			<div class="spinner-border" role="status">
				<span class="visually-hidden">{"Loading..."}</span>
			</div>
		},
		(true, false, Some(data)) => html! {<>
			{data.iter().map(|repo_meta| {
				html! {
					<div>
						{format!("{repo_meta:?}")}
					</div>
				}
			}).collect::<Vec<_>>()}
		</>},
		(true, false, None) => html!("data not available"),
	};
	html! {<>{content}</>}
}
