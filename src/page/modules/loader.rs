use crate::{
	database::app::Database,
	storage::{
		github::{
			ChangedFileStatus, CreateRepoArgs, FileContentArgs, FilesChangedArgs, GetTreeArgs, GithubClient,
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
	collections::{BTreeMap, HashSet, VecDeque},
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};
use wasm_bindgen_futures::spawn_local;

struct SignalWithOutput<V> {
	signal: task::Signal,
	output: Arc<Mutex<Option<V>>>,
}
impl<V> SignalWithOutput<V> {
	async fn output(self) -> Option<V> {
		self.signal.wait_true().await;
		let mut handle = self.output.lock().unwrap();
		handle.take()
	}
}

fn task_with_output<F, V, E>(
	task_dispatch: &task::Dispatch,
	name: impl Into<String>,
	progress: Option<task::ProgressHandle>,
	pending: F,
) -> SignalWithOutput<V>
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
	SignalWithOutput { signal, output }
}

// Query github for the logged in user and all organizations they have access to.
struct TaskQueryRepoOwners {
	client: GithubClient,
	on_missing_homebrew: Box<dyn FnOnce() -> LocalBoxFuture<'static, ()>>,
}
impl TaskQueryRepoOwners {
	async fn spawn(self, task_dispatch: &task::Dispatch) -> Option<Vec<String>> {
		let signal = task_with_output(task_dispatch, "Query Current User & Orgs", None, self.run());
		signal.output().await
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
		Self { client: client.clone() }
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
		let signal = task_with_output(
			task_dispatch,
			"Scan for Modules",
			None,
			self.run(),
		);
		signal.output().await
	}

	async fn run(self) -> anyhow::Result<Vec<RepositoryMetadata>> {
		use futures_util::stream::StreamExt;
		// Regardless of if the homebrew already existed, lets gather ALL of the relevant
		// repositories which are content modules. This will always include the homebrew repo,
		// since it is garunteed to exist due to the above code.
		let mut relevant_list = BTreeMap::new();
		let mut metadata = Vec::new();
		let mut stream = self.client.search_for_repos(self.owners.iter());
		while let Some(repos) = stream.next().await {
			metadata.extend(repos.clone());
			for repo in repos {
				relevant_list.insert((repo.owner, repo.name), (repo.is_private, repo.version));
			}
		}
		Ok(metadata)
	}
}

struct TaskScanRepository {
	client: GithubClient,
	repository: RepositoryMetadata,
	version_in_ddb: Option<String>,
	prev_update_signal: Option<task::Signal>,
}
struct ModuleUpdate {
	repository: RepositoryMetadata,
	generate_new_system_modules: bool,
	files: Vec<FileToUpdate>,
}
struct FileToUpdate {
	// The game system the file is in.
	system: String,
	// The path within the module of the file (including game system root).
	path_in_repo: String,
	// The file-id sha in the github repo.
	file_id: String,
	status: ChangedFileStatus,
}
impl TaskScanRepository {
	fn spawn(self, task_dispatch: &task::Dispatch) -> SignalWithOutput<ModuleUpdate> {
		let progress = task_dispatch.new_progress(1);
		task_with_output(
			task_dispatch,
			format!(
				"{} {}/{}",
				match &self.version_in_ddb {
					None => "Scanning for Download",
					Some(_) => "Scanning for Changes",
				},
				self.repository.owner,
				self.repository.name
			),
			Some(progress.clone()),
			self.run(progress),
		)
	}

	async fn run(self, progress: task::ProgressHandle) -> anyhow::Result<ModuleUpdate> {
		// Wait for the previous scan to finish (only 1 active scan is allowed at a time)
		if let Some(prev) = self.prev_update_signal {
			prev.wait_true().await;
		}
		match self.version_in_ddb {
			None => Self::scan_for_download(self.client, self.repository, progress).await,
			Some(version) => Self::scan_for_update(self.client, self.repository, version, progress).await,
		}
	}

	fn get_system_in_file_path(path: &std::path::Path) -> Option<String> {
		let Some(system_path) = path.components().next() else { return None; };
		let system = system_path.as_os_str().to_str().unwrap().to_owned();
		Some(system)
	}

	async fn scan_for_download(
		client: GithubClient,
		repository: RepositoryMetadata,
		mut progress: task::ProgressHandle,
	) -> anyhow::Result<ModuleUpdate> {
		let mut files = Vec::new();

		// Recursively scan the repository tree for all relevant content files
		let mut tree_ids = VecDeque::from([(PathBuf::new(), repository.tree_id.clone())]);
		while let Some((tree_path, tree_id)) = tree_ids.pop_front() {
			let args = GetTreeArgs {
				owner: repository.owner.as_str(),
				repo: repository.name.as_str(),
				tree_id: tree_id.as_str(),
			};
			for entry in client.get_tree(args).await? {
				let full_path = tree_path.join(&entry.path);
				// if the entry is a directory, put it in the queue to be scanned
				if entry.is_tree {
					tree_ids.push_back((full_path, entry.id));
					progress.inc_max(1);
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
					let path_in_repo = full_path.display().to_string().replace("\\", "/");
					files.push(FileToUpdate {
						system,
						path_in_repo,
						file_id: entry.id,
						status: ChangedFileStatus::Added,
					});
				}
			}
			progress.inc(1);
		}

		Ok(ModuleUpdate {
			repository,
			generate_new_system_modules: true,
			files,
		})
	}

	async fn scan_for_update(
		client: GithubClient,
		repository: RepositoryMetadata,
		version_in_ddb: String,
		mut progress: task::ProgressHandle,
	) -> anyhow::Result<ModuleUpdate> {
		// Getting the files changed for this upgrade
		let args = FilesChangedArgs {
			owner: repository.owner.as_str(),
			repo: repository.name.as_str(),
			commit_start: version_in_ddb.as_str(),
			commit_end: repository.version.as_str(),
		};
		let changed_file_paths = client.get_files_changed(args).await?;
		let mut files = Vec::with_capacity(changed_file_paths.len());
		for changed_file in changed_file_paths {
			let path_in_repo = std::path::Path::new(&changed_file.path);
			let system = Self::get_system_in_file_path(path_in_repo).unwrap();
			files.push(FileToUpdate {
				system,
				path_in_repo: changed_file.path,
				file_id: changed_file.file_id,
				status: changed_file.status,
			});
		}
		progress.inc(1);
		Ok(ModuleUpdate {
			repository,
			generate_new_system_modules: false,
			files,
		})
	}
}

struct TaskFetchModuleFiles {
	client: GithubClient,
	system_depot: system::Depot,
	repository: RepositoryMetadata,
	generate_new_system_modules: bool,
	files: Vec<FileToUpdate>,
	prev_update_signal: Option<task::Signal>,
}
struct ParsedModuleRecords {
	repository: RepositoryMetadata,
	generate_new_system_modules: bool,
	entries: Vec<crate::database::app::Entry>,
	removed_file_ids: HashSet<String>,
}
impl TaskFetchModuleFiles {
	fn spawn(self, task_dispatch: &task::Dispatch) -> SignalWithOutput<ParsedModuleRecords> {
		let progress = task_dispatch.new_progress(self.files.len() as u32);
		task_with_output(
			task_dispatch,
			format!("Downloading {}/{}", self.repository.owner, self.repository.name),
			Some(progress.clone()),
			self.run(progress),
		)
	}

	async fn run(mut self, mut progress: task::ProgressHandle) -> anyhow::Result<ParsedModuleRecords> {
		// Wait for the previous scan to finish (only 1 active scan is allowed at a time)
		if let Some(prev) = &self.prev_update_signal {
			prev.wait_true().await;
		}

		let mut entries = Vec::with_capacity(self.files.len());
		let mut removed_file_ids = HashSet::new();
		let files = self.files.drain(..).collect::<Vec<_>>();
		for file_to_update in files {
			let FileToUpdate {
				system,
				path_in_repo,
				file_id,
				status,
			} = file_to_update;
			let args = FileContentArgs {
				owner: self.repository.owner.as_str(),
				repo: self.repository.name.as_str(),
				path: Path::new(path_in_repo.as_str()),
				version: self.repository.version.as_str(),
			};
			match status {
				ChangedFileStatus::Added
				| ChangedFileStatus::Modified
				| ChangedFileStatus::Renamed
				| ChangedFileStatus::Copied
				| ChangedFileStatus::Changed => {
					let content = self.client.get_file_content(args).await?;
					let parsed_entries = self.parse_content(system, path_in_repo, file_id, content)?;
					entries.extend(parsed_entries);
				}
				ChangedFileStatus::Removed => {
					removed_file_ids.insert(file_id);
				}
				ChangedFileStatus::Unchanged => {}
			}
			progress.inc(1);
		}
		Ok(ParsedModuleRecords {
			repository: self.repository,
			generate_new_system_modules: self.generate_new_system_modules,
			entries,
			removed_file_ids,
		})
	}

	fn parse_content(
		&self,
		system: String,
		file_path: String,
		file_id: String,
		content: String,
	) -> anyhow::Result<Vec<crate::database::app::Entry>> {
		use anyhow::Context;
		let Some(system_reg) = self.system_depot.get(&system) else { return Ok(Vec::new()); };

		let document = content
			.parse::<kdl::KdlDocument>()
			.with_context(|| format!("{file_path}"))?;
		let path_in_system = match file_path.strip_prefix(&format!("{system}/")) {
			Some(systemless) => PathBuf::from(systemless),
			None => PathBuf::from(&file_path),
		};
		let mut source_id = SourceId {
			module: Some(self.repository.module_id()),
			system: Some(system.clone()),
			path: path_in_system,
			..Default::default()
		};
		let mut entries = Vec::with_capacity(document.nodes().len());
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let category = node.name().value().to_owned();
			let metadata = system_reg.parse_metadata(node, &source_id)?;
			let record = crate::database::app::Entry {
				id: source_id.to_string(),
				module: self.repository.module_id().to_string(),
				system: system.clone(),
				category: category,
				version: Some(self.repository.version.clone()),
				metadata,
				kdl: node.to_string(),
				file_id: Some(file_id.clone()),
			};
			entries.push(record);
		}
		Ok(entries)
	}
}

pub struct Loader {
	pub client: GithubClient,
	pub task_dispatch: task::Dispatch,
	pub system_depot: system::Depot,
	pub database: Database,
	pub on_finished: Box<dyn FnOnce()>,
}
impl Loader {
	pub fn find_and_download_modules(self) {
		spawn_local(Box::pin(async move {
			let task_create_homebrew = TaskCreateViewerHomebrew::new(&self.client);

			let task = TaskQueryRepoOwners {
				client: self.client.clone(),
				on_missing_homebrew: Box::new({
					let task_dispatch = self.task_dispatch.clone();
					move || Box::pin(task_create_homebrew.spawn(task_dispatch))
				}),
			};
			let Some(owners) = task.spawn(&self.task_dispatch).await else { return; };

			let task = TaskSearchForRelevantRepos {
				client: self.client.clone(),
				owners,
			};
			let Some(metadata) = task.spawn(&self.task_dispatch).await else { return; };

			let remote_updates = self.query_for_module_updates(metadata).await;
			let record_updates = self.fetch_updates_from_storage(remote_updates).await;
			for parsed_records in record_updates {
				let idb_result = self.commit_record_updates(parsed_records).await;
				if let Err(err) = idb_result {
					log::error!(target: "loader", "Failed to commit module records: {0}", err.to_string());
				}
			}

			(self.on_finished)();
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

	async fn query_for_module_updates(&self, metadata: Vec<RepositoryMetadata>) -> Vec<ModuleUpdate> {
		// Check each module to see what needs being updated.
		let mut prev_update_signal = None::<task::Signal>;
		let mut output_asyncs = Vec::with_capacity(metadata.len());
		for repo in metadata {
			let module_id = repo.module_id();
			// Check to see if the local info on this repository is out of sync with remote.
			let version_in_ddb = match self.get_ddb_module_version(&module_id).await {
				None => {
					// If a module doesn't exist in the database (repo owner + name as a module id),
					// then all of its content needs to be downloaded.
					None
				}
				Some(version_in_ddb) if version_in_ddb != repo.version => {
					// If a module DOES exist, but its version does not match the one in metadata,
					// then local data is out of date. Will need to query to see what files updated between the two versions,
					// and update individual entries based on that.
					Some(version_in_ddb)
				}
				// If the module exists and is up to date, then no updates are required.
				Some(_current_version) => continue,
			};

			// For each module which needs a fresh download or an incremental update,
			// wait on the previous scan (only 1 scan active at a time),
			// and cache the output signal for collection after all scans have been dispatched.
			let task = TaskScanRepository {
				client: self.client.clone(),
				repository: repo,
				version_in_ddb,
				prev_update_signal: prev_update_signal.take(),
			};
			let output_signal = task.spawn(&self.task_dispatch);
			prev_update_signal = Some(output_signal.signal.clone());
			output_asyncs.push(Box::pin(output_signal.output()));
		}

		// Await all pending scans and collect info on what specific objects require updates
		let mut updates = Vec::with_capacity(output_asyncs.len());
		for pending in output_asyncs {
			let Some(update) = pending.await else { continue; };
			updates.push(update);
		}

		updates
	}

	async fn fetch_updates_from_storage(&self, remote_updates: Vec<ModuleUpdate>) -> Vec<ParsedModuleRecords> {
		let mut prev_update_signal = None::<task::Signal>;
		let mut output_asyncs = Vec::with_capacity(remote_updates.len());
		for update in remote_updates {
			let ModuleUpdate {
				repository,
				generate_new_system_modules,
				files,
			} = update;
			let task = TaskFetchModuleFiles {
				client: self.client.clone(),
				system_depot: self.system_depot.clone(),
				repository,
				generate_new_system_modules,
				files,
				prev_update_signal: prev_update_signal.take(),
			};
			let output_signal = task.spawn(&self.task_dispatch);
			prev_update_signal = Some(output_signal.signal.clone());
			output_asyncs.push(Box::pin(output_signal.output()));
		}

		// Await all pending scans and collect info on what specific objects require updates
		let mut updates = Vec::with_capacity(output_asyncs.len());
		for pending in output_asyncs {
			let Some(update) = pending.await else { continue; };
			updates.push(update);
		}

		updates
	}

	async fn commit_record_updates(&self, module_records: ParsedModuleRecords) -> Result<(), crate::database::Error> {
		use crate::database::{
			app::{entry, Entry, Module},
			ObjectStoreExt, TransactionExt,
		};

		let ParsedModuleRecords {
			repository,
			generate_new_system_modules,
			entries,
			removed_file_ids,
		} = module_records;
		let module_id = repository.module_id();

		let transaction = self.database.write()?;
		let module_store = transaction.object_store_of::<Module>()?;
		let entry_store = transaction.object_store_of::<Entry>()?;

		for record in entries {
			entry_store.put_record(&record).await?;
		}
		// Delete entries by module and file-id
		let entry_ids_to_remove = {
			use futures_util::StreamExt;
			let idx_module = entry_store.index_of::<entry::Module>()?;
			let module = repository.module_id().to_string();
			let mut cursor = idx_module.open_cursor(Some(&entry::Module { module })).await?;
			let mut entry_ids_to_remove = Vec::with_capacity(removed_file_ids.len());
			while let Some(entry) = cursor.next().await {
				let Some(file_id) = &entry.file_id else { continue; };
				if removed_file_ids.contains(file_id) {
					entry_ids_to_remove.push(entry.id.clone());
				}
			}
			entry_ids_to_remove
		};
		for entry_id in entry_ids_to_remove {
			entry_store.delete_record(entry_id).await?;
		}

		if generate_new_system_modules {
			let record = crate::database::app::Module {
				id: module_id.clone(),
				name: module_id.to_string(),
				systems: repository.systems.iter().cloned().collect(),
				version: repository.version.clone(),
				remote_version: repository.version.clone(),
				installed: true,
			};
			module_store.put_record(&record).await?;
		} else {
			let mut module = module_store.get_record::<Module>(module_id.to_string()).await?.unwrap();
			module.version = repository.version;
			module_store.put_record(&module).await?;
		}

		transaction.commit().await?;

		Ok(())
	}
}
