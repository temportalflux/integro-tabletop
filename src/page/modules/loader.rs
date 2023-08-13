use crate::{
	database::app::Database,
	storage::{
		github::{
			CreateRepoArgs, FileContentArgs, FilesChangedArgs, GetTreeArgs, GithubClient,
			RepositoryMetadata, SetRepoTopicsArgs,
		},
		USER_HOMEBREW_REPO_NAME,
	},
	system::{
		self,
		core::{ModuleId, SourceId},
	},
	task,
};
use futures_util::future::LocalBoxFuture;
use std::{
	collections::{BTreeMap, VecDeque},
	path::PathBuf,
	sync::{Arc, Mutex},
};
use wasm_bindgen_futures::spawn_local;

async fn task_with_output<F, V, E>(
	task_dispatch: &task::Dispatch,
	name: impl Into<String>,
	progress: Option<task::ProgressHandle>,
	pending: F,
) -> Option<V>
where
	F: futures_util::Future<Output = Result<V, E>> + 'static,
	V: 'static,
	E: 'static,
	anyhow::Error: From<E>,
{
	let output = Arc::new(Mutex::new(None));
	let channel = output.clone();
	let signal = task_dispatch.spawn(name, progress, async move {
		let value = pending.await?;
		*channel.lock().unwrap() = Some(value);
		Ok(()) as anyhow::Result<(), E>
	});
	signal.wait_true().await;
	let mut handle = output.lock().unwrap();
	handle.take()
}

// Query github for the logged in user and all organizations they have access to.
struct TaskQueryRepoOwners {
	client: GithubClient,
	on_missing_homebrew: Box<dyn FnOnce() -> LocalBoxFuture<'static, ()>>,
}
impl TaskQueryRepoOwners {
	async fn spawn(self, task_dispatch: &task::Dispatch) -> Option<Vec<String>> {
		task_with_output(task_dispatch, "Query Current User & Orgs", None, self.run()).await
	}

	async fn run(self) -> anyhow::Result<Vec<String>> {
		use futures_util::stream::StreamExt;
		let (user, homebrew_repo) = self.client.viewer().await?;

		let mut owners = vec![user.clone()];
		let mut find_all_orgs = self.client.find_all_orgs();
		while let Some(org_list) = find_all_orgs.next().await {
			owners.extend(org_list);
		}

		// If the homebrew repo was not found when querying who the user is,
		// then we need to generate one, since this is where their user data is stored
		// and is the default location for any creations.
		if homebrew_repo.is_none() {
			(self.on_missing_homebrew)().await;
		}

		Ok(owners)
	}
}

// Create the homebrew repo on the github client viewer (the user that is logged in).
// https://docs.github.com/en/rest/repos/repos?apiVersion=2022-11-28#create-a-repository-for-the-authenticated-user
// https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28#create-or-update-file-contents
struct TaskCreateViewerHomebrew {
	client: GithubClient,
}
impl TaskCreateViewerHomebrew {
	fn new(client: &GithubClient) -> Self {
		Self {
			client: client.clone(),
		}
	}

	async fn spawn(self, task_dispatch: task::Dispatch) {
		let signal = task_dispatch.spawn("Initialize User Homebrew", None, self.run());
		signal.wait_true().await;
	}

	async fn run(self) -> anyhow::Result<()> {
		use crate::storage::github::MODULE_TOPIC;
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
	}
}

struct TaskSearchForRelevantRepos {
	client: GithubClient,
	owners: Vec<String>,
}
impl TaskSearchForRelevantRepos {
	async fn spawn(self, task_dispatch: &task::Dispatch) -> Option<Vec<RepositoryMetadata>> {
		let mut progress = task_dispatch.new_progress(self.owners.len() as u32);
		task_with_output(
			task_dispatch,
			"Scan for Modules",
			Some(progress.clone()),
			self.run(progress),
		)
		.await
	}

	async fn run(self, mut progress: task::ProgressHandle) -> anyhow::Result<Vec<RepositoryMetadata>> {
		use futures_util::stream::StreamExt;
		// Regardless of if the homebrew already existed, lets gather ALL of the relevant
		// repositories which are content modules. This will always include the homebrew repo,
		// since it is garunteed to exist due to the above code.
		let mut relevant_list = BTreeMap::new();
		let mut metadata = Vec::new();
		for owner in self.owners {
			let mut stream = self.client.search_for_repos(&owner);
			while let Some(repos) = stream.next().await {
				metadata.extend(repos.clone());
				for repo in repos {
					relevant_list.insert((repo.owner, repo.name), (repo.is_private, repo.version));
				}
			}
			progress.inc(1);
		}

		Ok(metadata)
	}
}

#[derive(Clone)]
pub struct Loader {
	pub client: GithubClient,
	pub task_dispatch: task::Dispatch,
	pub system_depot: system::Depot,
	pub database: Database,
}
impl Loader {
	pub fn find_and_download_modules(self) {
		spawn_local(Box::pin(async move {
			let task_create_homebrew = TaskCreateViewerHomebrew::new(&self.client);

			let task = TaskQueryRepoOwners {
				client: self.client.clone(),
				on_missing_homebrew: Box::new({
					let task_dispatch = self.task_dispatch.clone();
					move || {
						Box::pin(task_create_homebrew.spawn(task_dispatch))
					}
				}),
			};
			let Some(owners) = task.spawn(&self.task_dispatch).await else { return; };

			let task = TaskSearchForRelevantRepos { client: self.client.clone(), owners };
			let Some(metadata) = task.spawn(&self.task_dispatch).await else { return; };

			self.insert_or_update_modules(metadata);
		}));
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
		let mut progress = self.task_dispatch.new_progress(repositories.len() as u32);
		self.task_dispatch.clone().spawn(
			"Check for Module Updates",
			Some(progress.clone()),
			async move {
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
							let signal =
								self.download_module(repo, module_id, prev_update_signal.take());
							prev_update_signal = Some(signal);
						}
						Some(version_in_ddb) if version_in_ddb != repo.version => {
							// If a module DOES exist, but its version does not match the one in metadata,
							// then local data is out of date. Will need to query to see what files updated between the two versions,
							// and update individual entries based on that.
							let signal = self.update_module(
								repo,
								module_id,
								version_in_ddb,
								prev_update_signal.take(),
							);
							prev_update_signal = Some(signal);
						}
						// If the module exists and is up to date, then no updates are required.
						Some(_current_version) => {}
					}
					progress.inc(1);
				}

				Ok(()) as anyhow::Result<()>
			},
		);
	}

	fn get_system_in_file_path(path: &std::path::Path) -> Option<String> {
		let Some(system_path) = path.components().next() else { return None; };
		let system = system_path.as_os_str().to_str().unwrap().to_owned();
		Some(system)
	}

	fn download_module(
		&self,
		repo: RepositoryMetadata,
		module_id: ModuleId,
		prev_signal: Option<task::Signal>,
	) -> task::Signal {
		use crate::database::{
			app::{Entry, Module},
			ObjectStoreExt, TransactionExt,
		};
		let mut scan_progress = self.task_dispatch.new_progress(1);
		self.task_dispatch.spawn(
			format!("Scanning {}/{}", repo.owner, repo.name),
			Some(scan_progress.clone()),
			{
				let client = self.client.clone();
				let system_depot = self.system_depot.clone();
				let database = self.database.clone();
				async move {
					if let Some(prev) = prev_signal {
						prev.wait_true().await;
					}

					let mut file_paths = Vec::new();

					// Recursively scan the repository tree for all relevant content files
					let mut tree_ids = VecDeque::from([(PathBuf::new(), repo.tree_id.clone())]);
					while let Some((tree_path, tree_id)) = tree_ids.pop_front() {
						let args = GetTreeArgs {
							owner: repo.owner.as_str(),
							repo: repo.name.as_str(),
							tree_id: tree_id.as_str(),
						};
						for entry in client.get_tree(args).await? {
							let full_path = tree_path.join(&entry.path);
							// if the entry is a directory, put it in the queue to be scanned
							if entry.is_tree {
								tree_ids.push_back((full_path, entry.id));
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
									Some(path) if path == std::path::Path::new("") => continue,
									_ => {}
								}
								let system = Self::get_system_in_file_path(&full_path).unwrap();
								let path_str = full_path.display().to_string().replace("\\", "/");
								file_paths.push((system, path_str, entry.id));
							}
						}
						scan_progress.inc(1);
					}

					// Prepare the module to be inserted into the database
					let db_modules = {
						let mut modules = Vec::with_capacity(repo.systems.len());
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
					for (system, file_path, file_id) in file_paths {
						let args = FileContentArgs {
							owner: repo.owner.as_str(),
							repo: repo.name.as_str(),
							path: file_path.as_str(),
							version: repo.version.as_str(),
						};
						let content = client.get_file_content(args).await?;

						let entries = Self::parse_content(
							&system_depot,
							&repo,
							&module_id,
							system,
							file_path,
							file_id,
							content,
						);
						db_entries.extend(entries);

						scan_progress.inc(1);
					}

					// Now that we have all of the entries to insert,
					// put them all in the database.
					{
						let transaction = database.write()?;
						let module_store = transaction.object_store_of::<Module>()?;
						let entry_store = transaction.object_store_of::<Entry>()?;
						for record in db_modules {
							module_store.add_record(&record).await?;
						}
						for record in db_entries {
							entry_store.add_record(&record).await?;
						}
						transaction.commit().await?;
					}

					Ok(()) as Result<(), LoaderError>
				}
			},
		)
	}

	fn update_module(
		&self,
		repo: RepositoryMetadata,
		module_id: ModuleId,
		version_in_ddb: String,
		prev_signal: Option<task::Signal>,
	) -> task::Signal {
		use crate::database::{
			app::{Entry, Module},
			ObjectStoreExt, TransactionExt,
		};
		let mut progress = self.task_dispatch.new_progress(1);
		self.task_dispatch.spawn(
			format!("Updating {}/{}", repo.owner, repo.name),
			Some(progress.clone()),
			{
				let client = self.client.clone();
				let system_depot = self.system_depot.clone();
				let database = self.database.clone();
				async move {
					if let Some(prev) = prev_signal {
						prev.wait_true().await;
					}

					// Getting the files changed for this upgrade
					let changed_file_paths = client
						.get_files_changed(FilesChangedArgs {
							owner: repo.owner.as_str(),
							repo: repo.name.as_str(),
							commit_start: version_in_ddb.as_str(),
							commit_end: repo.version.as_str(),
						})
						.await?;

					progress.inc_max(changed_file_paths.len() as u32);
					let mut new_entries = Vec::with_capacity(changed_file_paths.len());

					for (file_path, file_id) in changed_file_paths {
						// Getting the content of each changed file
						let content = client
							.get_file_content(FileContentArgs {
								owner: repo.owner.as_str(),
								repo: repo.name.as_str(),
								path: file_path.as_str(),
								version: repo.version.as_str(),
							})
							.await?;
						let system =
							Self::get_system_in_file_path(std::path::Path::new(&file_path))
								.unwrap();

						let entries = Self::parse_content(
							&system_depot,
							&repo,
							&module_id,
							system,
							file_path,
							file_id,
							content,
						);
						new_entries.extend(entries);
						progress.inc(1);
					}

					// After fetching all of the changes, update (add or replace) the entry records and update the module.
					{
						let transaction = database.write()?;
						let module_store = transaction.object_store_of::<Module>()?;
						let entry_store = transaction.object_store_of::<Entry>()?;

						let mut module = module_store
							.get_record::<Module>(module_id.to_string())
							.await?
							.unwrap();
						module.version = repo.version;
						module_store.put_record(&module).await?;

						for entry in new_entries {
							entry_store.put_record(&entry).await?;
						}

						transaction.commit().await?;
					}
					Ok(()) as Result<(), LoaderError>
				}
			},
		)
	}

	fn parse_content(
		system_depot: &system::Depot,
		repo: &RepositoryMetadata,
		module_id: &ModuleId,
		system: String,
		file_path: String,
		file_id: String,
		content: String,
	) -> Vec<crate::database::app::Entry> {
		let Some(system_reg) = system_depot.get(&system) else { return Vec::new(); };

		let document = match content.parse::<kdl::KdlDocument>() {
			Ok(doc) => doc,
			Err(err) => {
				log::warn!(
					"Failed to parse module content {}:{}/{}: {err:?}",
					repo.owner,
					repo.name,
					file_path
				);
				return Vec::new();
			}
		};
		let path_in_system = match file_path.strip_prefix(&format!("{system}/")) {
			Some(systemless) => PathBuf::from(systemless),
			None => PathBuf::from(&file_path),
		};
		let mut source_id = SourceId {
			module: Some(module_id.clone()),
			system: Some(system.clone()),
			path: path_in_system,
			..Default::default()
		};
		let mut entries = Vec::with_capacity(document.nodes().len());
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let category = node.name().value().to_owned();
			let metadata = match system_reg.parse_metadata(node, &source_id) {
				Ok(metadata) => metadata,
				Err(err) => {
					log::error!("{err:?}");
					continue;
				}
			};
			let record = crate::database::app::Entry {
				id: source_id.to_string(),
				module: module_id.to_string(),
				system: system.clone(),
				category: category,
				version: Some(repo.version.clone()),
				metadata,
				kdl: node.to_string(),
				file_id: Some(file_id.clone()),
			};
			entries.push(record);
		}
		entries
	}
}

#[derive(thiserror::Error, Debug, Clone)]
enum LoaderError {
	#[error("{0}")]
	ReqwestError(String),
	#[error("{0}")]
	IndexedDBError(String),
	#[error(transparent)]
	DatabaseError(#[from] crate::database::Error),
}
impl From<idb::Error> for LoaderError {
	fn from(value: idb::Error) -> Self {
		Self::IndexedDBError(value.to_string())
	}
}
impl From<reqwest::Error> for LoaderError {
	fn from(value: reqwest::Error) -> Self {
		Self::ReqwestError(value.to_string())
	}
}
