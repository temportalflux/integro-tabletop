use crate::{
	auth,
	storage::github::{CreateRepo, GithubClient, SetRepoTopics},
	task,
};
use std::{collections::BTreeMap, rc::Rc};
use yew::prelude::*;
use yewdux::prelude::*;

/// Page which displays the modules the user currently logged in has contributor access to.
#[function_component]
pub fn OwnedModules() -> Html {
	let task_dispatch = use_context::<task::Dispatch>().unwrap();
	let (auth_status, _) = use_store::<auth::Status>();
	let was_authed = use_state_eq(|| false);
	let relevant_repos = use_state(|| BTreeMap::<(String, String), (bool, String)>::new());

	let signed_in = matches!(*auth_status, auth::Status::Successful { .. });
	if *was_authed && !signed_in {
		was_authed.set(false);
		relevant_repos.set(BTreeMap::default());
	} else if !*was_authed && signed_in {
		was_authed.set(true);
		if let Some(post_login) = PostLogin::new(&task_dispatch, &auth_status, &relevant_repos) {
			post_login.query_user_and_orgs();
		}
	}

	let content = match (signed_in, &*relevant_repos) {
		(false, _) => html!("Not signed in"),
		(true, data) if data.len() == 0 => html! {
			<div class="spinner-border" role="status">
				<span class="visually-hidden">{"Loading..."}</span>
			</div>
		},
		(true, data) => html! {<>
			{data.iter().map(|((owner, name), (is_private, version))| {
				html! {
					<div>
						{format!("{owner}/{name}")}
						{is_private.then(|| html!(" [private]")).unwrap_or_default()}
						{format!(" - version: {version}")}
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

static USER_HOMEBREW_REPO_NAME: &str = "integro-homebrew";

#[derive(Clone)]
struct PostLogin {
	task_dispatch: task::Dispatch,
	client: GithubClient,
	relevant_repos: UseStateHandle<BTreeMap<(String, String), (bool, String)>>,
}
impl PostLogin {
	fn new(
		task_dispatch: &task::Dispatch,
		auth_status: &Rc<auth::Status>,
		relevant_repos: &UseStateHandle<BTreeMap<(String, String), (bool, String)>>,
	) -> Option<Self> {
		let auth::Status::Successful { token } = &**auth_status else { return None; };
		log::debug!("detected login {token:?}");
		let Ok(client) = GithubClient::new(token) else { return None; };
		Some(Self {
			client,
			task_dispatch: task_dispatch.clone(),
			relevant_repos: relevant_repos.clone(),
		})
	}

	fn query_user_and_orgs(self) {
		use futures_util::stream::StreamExt;
		self.task_dispatch
			.clone()
			.spawn("Query Current User & Orgs", None, {
				async move {
					let user = self.client.viewer().await?.viewer.login;

					let mut owners = vec![user.clone()];
					let mut find_all_orgs = self.client.find_all_orgs();
					while let Some(org_list) = find_all_orgs.next().await {
						owners.extend(org_list);
					}
					log::debug!("{owners:?}");

					self.search_for_relevant_repos(user, owners);

					Ok(())
				}
			});
	}

	fn search_for_relevant_repos(self, user: String, owners: Vec<String>) {
		use futures_util::stream::StreamExt;
		let mut progress = self.task_dispatch.new_progress(owners.len() as u32);
		self.task_dispatch
			.clone()
			.spawn("Scan for Modules", Some(progress.clone()), async move {
				// First query to see if the user has a homebrew module.
				// All users need one, as this is where their user data is stored
				// and is the default location for any creations.
				let homebrew_repo = self.client.find_user_repo(user.clone(), USER_HOMEBREW_REPO_NAME.to_owned()).await?;
				if homebrew_repo.is_none() {
					// If there is no pre-existing homebrew repo, create one.
					self.clone().create_user_homebrew().wait_true().await;
				}
				
				// Regardless of if the homebrew already existed, lets gather ALL of the relevant
				// repositories which are content modules. This will always include the homebrew repo,
				// since it is garunteed to exist due to the above code.
				let mut relevant_list = BTreeMap::new();
				for owner in &owners {
					log::debug!("searching {owner:?}");
					let mut stream = self.client.search_for_repos(owner);
					while let Some(repos) = stream.next().await {
						for repo in repos {
							relevant_list
								.insert((repo.owner, repo.name), (repo.is_private, repo.version));
						}
					}
					progress.inc(1);
				}
				log::debug!("Valid Repositories: {relevant_list:?}");

				self.relevant_repos.set(relevant_list);
				Ok(())
			});
	}

	fn create_user_homebrew(self) -> task::Signal {
		use crate::storage::github::MODULE_TOPIC;
		// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
		// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#create-or-update-file-contents
		self.task_dispatch
			.clone()
			.spawn("Initialize User Homebrew", None, async move {
				let create_repo = CreateRepo {
					org: None,
					name: USER_HOMEBREW_REPO_NAME.to_owned(),
					private: true,
				};
				let owner = self.client.create_repo(create_repo).await?;

				let set_topics = SetRepoTopics {
					owner,
					repo: USER_HOMEBREW_REPO_NAME.to_owned(),
					topics: vec![MODULE_TOPIC.to_owned()],
				};
				self.client.set_repo_topics(set_topics).await?;

				Ok(())
			})
	}
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
