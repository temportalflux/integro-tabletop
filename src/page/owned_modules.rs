use crate::{
	auth,
	database::app::Database,
	kdl_ext::NodeContext,
	storage::{
		github::{
			CreateRepoArgs, FileContentArgs, FilesChangedArgs, GetTreeArgs, GithubClient,
			RepositoryMetadata, SetRepoTopicsArgs,
		},
		USER_HOMEBREW_REPO_NAME,
	},
	system::core::{ModuleId, SourceId},
	task,
};
use anyhow::Context;
use std::{
	collections::{BTreeMap, VecDeque},
	path::PathBuf,
	rc::Rc,
	sync::Arc,
};
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
				let create_repo = CreateRepoArgs {
					org: None,
					name: USER_HOMEBREW_REPO_NAME,
					private: true,
				};
				let owner = self.client.create_repo(create_repo).await?;

				let set_topics = SetRepoTopicsArgs {
					owner: owner.as_str(),
					repo: USER_HOMEBREW_REPO_NAME,
					topics: vec![MODULE_TOPIC.to_owned()],
				};
				self.client.set_repo_topics(set_topics).await?;

				Ok(())
			})
	}

	async fn get_ddb_module_version(&self, id: &ModuleId) -> Option<String> {
		use crate::database::app::Module;
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
		use crate::database::{
			app::{Entry, Module},
			ObjectStoreExt, TransactionExt,
		};
		let mut progress = self.task_dispatch.new_progress(repositories.len() as u32);
		self.task_dispatch.clone().spawn(
			"Check for Module Updates",
			Some(progress.clone()),
			async move {
				// TODO: Check the database to see what data needs to be updated.

				// TODO: These need to go somewhere that the registries are both:
				// - available via use_context
				// - holds comps and nodes for all systems (not just 1 at a time)
				let comp_reg = Rc::new(crate::system::dnd5e::component_registry());
				let node_reg = Arc::new(crate::system::dnd5e::node_registry());

				let mut prev_update_signal = None::<task::Signal>;
				for repo in repositories {
					let module_id = ModuleId::Github {
						user_org: repo.owner.clone(),
						repository: repo.name.clone(),
					};
					match self.get_ddb_module_version(&module_id).await {
						None => {
							// If a module doesn't exist in the database (repo owner + name as a module id),
							// then all of its content needs to be downloaded.

							log::debug!("TODO: full module download {}/{}", repo.owner, repo.name);
							// TEMPORARY guard to only fetch basic rules
							if repo.name != "dnd5e-basic-rules" {
								continue;
							}

							let mut scan_progress = self.task_dispatch.new_progress(1);
							let scan_signal = self.task_dispatch.spawn(
								format!("Scanning {}/{}", repo.owner, repo.name),
								Some(scan_progress.clone()),
								{
									let client = self.client.clone();
									let database = self.database.clone();
									let comp_reg = comp_reg.clone();
									let node_reg = node_reg.clone();
									let prev_update_signal = prev_update_signal.take();
									async move {
										if let Some(prev) = prev_update_signal {
											prev.wait_true().await;
										}

										let mut file_paths = Vec::new();

										let mut tree_ids = VecDeque::from([(
											PathBuf::new(),
											repo.tree_id.clone(),
										)]);
										while let Some((tree_path, tree_id)) = tree_ids.pop_front()
										{
											let args = GetTreeArgs {
												owner: repo.owner.as_str(),
												repo: repo.name.as_str(),
												tree_id: tree_id.as_str(),
											};
											for entry in client.get_tree(args).await? {
												let full_path = tree_path.join(&entry.path);
												if let Some(tree_id) = entry.tree_id {
													tree_ids.push_back((full_path, tree_id));
													scan_progress.inc_max(1);
												} else {
													// only record content files (kdl extension)
													if !entry.path.ends_with(".kdl") {
														continue;
													}
													// extract the system the content is for (which is the top-most parent).
													// if this path has no parent, then it isn't in a system and can be ignored.
													match full_path.parent() {
														None => continue,
														Some(path)
															if path == std::path::Path::new("") =>
														{
															continue
														}
														_ => {}
													}
													let mut comps = full_path.components();
													let system_path = comps.next().unwrap();
													let system = system_path
														.as_os_str()
														.to_str()
														.unwrap()
														.to_owned();
													let path_str = full_path
														.display()
														.to_string()
														.replace("\\", "/");
													file_paths.push((system, path_str));
												}
											}
											scan_progress.inc(1);
										}

										// Prepare the module to be inserted into the database
										let db_modules = {
											let mut modules =
												Vec::with_capacity(repo.systems.len());
											for system in &repo.systems {
												modules.push(Module {
													module_id: module_id.clone(),
													name: module_id.to_string(),
													system: system.clone(),
													version: repo.version.clone(),
												});
											}
											modules
										};
										let mut db_entries = Vec::with_capacity(file_paths.len());

										scan_progress.set_name(
											format!("Downloading {}/{}", repo.owner, repo.name),
											0,
											file_paths.len() as u32,
										);
										for (system, file_path) in file_paths {
											let args = FileContentArgs {
												owner: repo.owner.as_str(),
												repo: repo.name.as_str(),
												path: file_path.as_str(),
												version: repo.version.as_str(),
											};
											let content = client.get_file_content(args).await?;

											let document =
												match content.parse::<kdl::KdlDocument>() {
													Ok(doc) => doc,
													Err(err) => {
														log::warn!("Failed to parse module content {}:{}/{}: {err:?}", repo.owner, repo.name, file_path);
														continue;
													}
												};
											let mut source_id = SourceId {
												module: Some(module_id.clone()),
												system: Some(system.clone()),
												path: PathBuf::from(&file_path),
												version: None,
												node_idx: 0,
											};

											for (idx, node) in document.nodes().iter().enumerate() {
												source_id.node_idx = idx;

												let category = node.name().value().to_owned();
												// Generate the metadata for this entry
												let metadata = {
													let Some(comp_factory) = comp_reg.get_factory(&category).cloned() else { continue; };
													let ctx = NodeContext::new(
														Arc::new(source_id.clone()),
														node_reg.clone(),
													);
													let metadata = comp_factory
														.metadata_from_kdl(node, &ctx)
														.with_context(|| {
															format!(
																"Failed to parse {:?}",
																source_id.to_string()
															)
														});
													let metadata = match metadata {
														Ok(metadata) => metadata,
														Err(err) => {
															log::error!(
																"Failed to parse content metadata: {err:?}"
															);
															continue;
														}
													};
													match metadata {
														serde_json::Value::Null => {
															serde_json::Value::Null
														}
														serde_json::Value::Object(mut metadata) => {
															metadata.insert(
																"module".into(),
																serde_json::json!(
																	module_id.to_string()
																),
															);
															serde_json::Value::Object(metadata)
														}
														other => {
															log::error!(
																"Metadata must be a map, but {} returned {:?}.",
																source_id.to_string(),
																other
															);
															continue;
														}
													}
												};

												let record = Entry {
													id: source_id.to_string(),
													module: module_id.to_string(),
													system: system.clone(),
													category: category,
													version: Some(repo.version.clone()),
													metadata,
													kdl: node.to_string(),
												};
												db_entries.push(record);
											}

											scan_progress.inc(1);
										}

										// Now that we have all of the entries to insert,
										// put them all in the database.
										{
											let transaction = database.write()?;
											let module_store =
												transaction.object_store_of::<Module>()?;
											let entry_store =
												transaction.object_store_of::<Entry>()?;
											for record in db_modules {
												module_store.add_record(&record).await?;
											}
											for record in db_entries {
												entry_store.add_record(&record).await?;
											}
											if let Err(err) = transaction.commit().await {
												log::error!("Failed to commit module content for {}/{}: {err:?}", repo.owner, repo.name);
											}
										}

										Ok(())
									}
								},
							);
							prev_update_signal = Some(scan_signal);
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

							// TODO: After fetching all of the changes, remove the old records and insert the new ones. Also update the module version.

							/*
							// Getting the files changed for this upgrade
							let file_paths = self.client.get_files_changed(FilesChangedArgs {
								owner: "flux-tabletop".into(),
								repo: "dnd5e-basic-rules".into(),
								commit_start: "7b410f785b909d2c5569c3bbf896509cc74c402c".into(),
								commit_end: "eb0f3fe6bf1d6d44837d3339dafd80cceea24a16".into(),
							}).await?;

							// Getting the content of each changed file
							let content = self.client.get_file_content(FileContentArgs {
								owner: "flux-tabletop".into(),
								repo: "dnd5e-basic-rules".into(),
								path: "dnd5e/class/monk.kdl".into(),
								version: "eb0f3fe6bf1d6d44837d3339dafd80cceea24a16".into(),
							}).await?;
							*/
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
