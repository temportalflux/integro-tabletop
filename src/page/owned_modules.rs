use crate::{
	auth,
	storage::github::{GithubClient, RepositoryMetadata},
	task,
};
use std::rc::Rc;
use yew::prelude::*;
use yewdux::prelude::*;

/// Page which displays the modules the user currently logged in has contributor access to.
#[function_component]
pub fn OwnedModules() -> Html {
	let task_dispatch = use_context::<task::Dispatch>().unwrap();
	let (auth_status, _) = use_store::<auth::Status>();
	let was_authed = use_state_eq(|| false);
	let relevant_repos = use_state(|| None::<Vec<RepositoryMetadata>>);

	let signed_in = matches!(*auth_status, auth::Status::Successful { .. });
	if *was_authed && !signed_in {
		was_authed.set(false);
		relevant_repos.set(None);
	} else if !*was_authed && signed_in {
		was_authed.set(true);
		query_user_and_orgs(&task_dispatch, &auth_status, &relevant_repos);
	}

	let content = match (signed_in, &*relevant_repos) {
		(false, _) => html!("Not signed in"),
		(true, None) => html! {
			<div class="spinner-border" role="status">
				<span class="visually-hidden">{"Loading..."}</span>
			</div>
		},
		(true, Some(data)) => html! {<>
			{data.iter().map(|repo_meta| {
				html! {
					<div>
						{format!("{repo_meta:?}")}
					</div>
				}
			}).collect::<Vec<_>>()}
		</>},
	};
	html! {<>
		<TaskListView />
		{content}
	</>}
}

fn query_user_and_orgs(
	task_dispatch: &task::Dispatch,
	auth_status: &Rc<auth::Status>,
	relevant_repos: &UseStateHandle<Option<Vec<RepositoryMetadata>>>,
) {
	use futures_util::stream::StreamExt;
	task_dispatch.spawn("Query Current User & Orgs", None, {
		let auth_status = auth_status.clone();
		let task_dispatch = task_dispatch.clone();
		let relevant_repos = relevant_repos.clone();
		async move {
			let auth::Status::Successful { token } = &*auth_status else { return Ok(()); };
			log::debug!("detected login {token:?}");
			let client = GithubClient::new(token)?;
			let user = client.viewer().await?.viewer.login;

			let mut owners = vec![user];
			let mut find_all_orgs = client.find_all_orgs();
			while let Some(org_list) = find_all_orgs.next().await {
				owners.extend(org_list);
			}
			log::debug!("{owners:?}");

			search_for_relevant_repos(&task_dispatch, client, owners, relevant_repos);

			Ok(())
		}
	});
}

fn search_for_relevant_repos(
	task_dispatch: &task::Dispatch,
	client: GithubClient,
	owners: Vec<String>,
	relevant_repos: UseStateHandle<Option<Vec<RepositoryMetadata>>>,
) {
	use futures_util::stream::StreamExt;
	let mut progress = task_dispatch.new_progress(owners.len() as u32);
	task_dispatch.spawn("Scan for Modules", Some(progress.clone()), async move {
		let mut relevant_list = Vec::new();
		for owner in &owners {
			log::debug!("searching {owner:?}");
			let mut stream = client.search_for_repos(owner);
			while let Some(repos) = stream.next().await {
				relevant_list.extend(repos);
			}
			progress.inc(1);
		}
		log::debug!("Valid Repositories: {relevant_list:?}");
		relevant_repos.set(Some(relevant_list));
		Ok(())
	});
}

#[function_component]
pub fn TaskListView() -> Html {
	let task_view = use_context::<task::View>().unwrap();
	html! {
		<div>
			{task_view.iter().map(|handle| html! {
				<div class="d-flex align-items-center">
					<span class="me-1">{&handle.name}{":"}</span>
					{match &handle.status {
						task::Status::Pending => {
							html! {
								<span>
									{"PENDING"}
									{handle.progress.as_ref().map(|(value, max)| {
										html!(format!(" ({value} / {max})"))
									}).unwrap_or_default()}
								</span>
							}
						}
						task::Status::Failed(error) => {
							html!(<span>{format!("FAILED: {error:?}")}</span>)
						}
					}}
				</div>
			}).collect::<Vec<_>>()}
		</div>
	}
}
