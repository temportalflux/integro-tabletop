use crate::{
	auth,
	database::app::Database,
	storage::{
		github::{CreateRepo, GithubClient, RepositoryMetadata, SetRepoTopics},
		USER_HOMEBREW_REPO_NAME,
	},
	system::core::ModuleId,
	task,
};
use std::{collections::BTreeMap, rc::Rc};
use yew::prelude::*;
use yewdux::prelude::*;

/// Page which displays the modules the user currently logged in has contributor access to.
#[function_component]
pub fn OwnedModules() -> Html {
	let database = use_context::<Database>().unwrap();
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
		if let Some(post_login) =
			PostLogin::new(&task_dispatch, &auth_status, &database, &relevant_repos)
		{
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

#[derive(Clone)]
struct PostLogin {
	client: GithubClient,
	task_dispatch: task::Dispatch,
	database: Database,
	relevant_repos: UseStateHandle<BTreeMap<(String, String), (bool, String)>>,
}
impl PostLogin {
	fn new(
		task_dispatch: &task::Dispatch,
		auth_status: &Rc<auth::Status>,
		database: &Database,
		relevant_repos: &UseStateHandle<BTreeMap<(String, String), (bool, String)>>,
	) -> Option<Self> {
		let auth::Status::Successful { token } = &**auth_status else { return None; };
		log::debug!("detected login {token:?}");
		let Ok(client) = GithubClient::new(token) else { return None; };
		Some(Self {
			client,
			task_dispatch: task_dispatch.clone(),
			database: database.clone(),
			relevant_repos: relevant_repos.clone(),
		})
	}

	fn query_user_and_orgs(self) {
		use futures_util::stream::StreamExt;
		self.task_dispatch
			.clone()
			.spawn("Query Current User & Orgs", None, {
				async move {
					let (user, homebrew_repo) = self.client.viewer().await?;

					let mut owners = vec![user.clone()];
					let mut find_all_orgs = self.client.find_all_orgs();
					while let Some(org_list) = find_all_orgs.next().await {
						owners.extend(org_list);
					}
					log::debug!("{owners:?}");

					// If the homebrew repo was not found when querying who the user is,
					// then we need to generate one, since this is where their user data is stored
					// and is the default location for any creations.
					if homebrew_repo.is_none() {
						self.clone().create_user_homebrew().wait_true().await;
					}

					self.search_for_relevant_repos(owners);

					Ok(())
				}
			});
	}

	fn search_for_relevant_repos(self, owners: Vec<String>) {
		use futures_util::stream::StreamExt;
		let mut progress = self.task_dispatch.new_progress(owners.len() as u32);
		self.task_dispatch
			.clone()
			.spawn("Scan for Modules", Some(progress.clone()), async move {
				// Regardless of if the homebrew already existed, lets gather ALL of the relevant
				// repositories which are content modules. This will always include the homebrew repo,
				// since it is garunteed to exist due to the above code.
				let mut relevant_list = BTreeMap::new();
				let mut metadata = Vec::new();
				for owner in &owners {
					log::debug!("searching {owner:?}");
					let mut stream = self.client.search_for_repos(owner);
					while let Some(repos) = stream.next().await {
						metadata.extend(repos.clone());
						for repo in repos {
							relevant_list
								.insert((repo.owner, repo.name), (repo.is_private, repo.version));
						}
					}
					progress.inc(1);
				}
				log::debug!("Valid Repositories: {relevant_list:?}");

				self.relevant_repos.set(relevant_list);
				self.insert_or_update_modules(metadata);

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

	async fn get_ddb_module_version(&self, repo: &RepositoryMetadata) -> Option<String> {
		use crate::database::app::Module;
		let id = ModuleId::Github {
			user_org: repo.owner.clone(),
			repository: repo.name.clone(),
		};
		let query = self.database.get::<Module>(id.to_string()).await;
		match query {
			Ok(module) => module.map(|module| module.version),
			Err(err) => {
				log::warn!("Query for {id:?} failed: {err:?}");
				None
			}
		}
	}

	fn insert_or_update_modules(self, repositories: Vec<RepositoryMetadata>) {
		let mut progress = self.task_dispatch.new_progress(repositories.len() as u32);
		self.task_dispatch.clone().spawn(
			"Check for Module Updates",
			Some(progress.clone()),
			async move {
				// TODO: Check the database to see what data needs to be updated.

				for repo in repositories {
					match self.get_ddb_module_version(&repo).await {
						None => {
							// If a module doesn't exist in the database (repo owner + name as a module id),
							// then all of its content needs to be downloaded.
							log::debug!("TODO: full module download {}/{}", repo.owner, repo.name);
						}
						Some(version) if version != repo.version => {
							// If a module DOES exist, but its version does not match the one in metadata,
							// then local data is out of date. Will need to query to see what files updated between the two versions,
							// and update individual entries based on that.
							log::debug!(
								"TODO: partial download {}/{} ({} -> {})",
								repo.owner,
								repo.name,
								version,
								repo.version
							);
						}
						// If the module exists and is up to date, then no updates are required.
						Some(_current_version) => {
							log::debug!(
								"TODO: up-to-date {}/{} ({_current_version})",
								repo.owner,
								repo.name
							);
						}
					}
					progress.inc(1);
				}

				Ok(())
			},
		);
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
