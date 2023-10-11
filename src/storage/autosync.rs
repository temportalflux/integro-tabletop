use crate::{
	database::{
		self,
		app::{Database, Module},
	},
	storage::github::{ChangedFileStatus, RepositoryMetadata},
	system::core::{ModuleId, SourceId},
};
use std::{
	collections::{BTreeMap, HashMap},
	rc::Rc, cell::RefCell,
};
use derivative::Derivative;
use yew::{html::ChildrenProps, prelude::*};
use yew_hooks::*;

mod download_file_updates;
use download_file_updates::*;
mod find_file_updates;
use find_file_updates::*;
mod find_modules;
use find_modules::*;
mod generate_homebrew;
use generate_homebrew::*;
mod query_module_owners;
use query_module_owners::*;
mod scan_for_modules;
use scan_for_modules::*;
mod scan_repository;
use scan_repository::*;

#[derive(Clone)]
pub struct Channel(Rc<RequestChannel>);
impl PartialEq for Channel {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for Channel {
	type Target = RequestChannel;

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

pub struct RequestChannel {
	send_req: async_channel::Sender<Request>,
	recv_req: async_channel::Receiver<Request>,
}
impl RequestChannel {
	pub fn try_send_req(&self, req: Request) {
		let _ = self.send_req.try_send(req);
	}
}

#[derive(Debug)]
pub enum Request {
	// Only poll for what the latest version is of all installed modules.
	// This should not actually download any updates.
	FetchLatestVersionAllModules,
	// Polls what the latest version is for each provided module,
	// queuing downloads for each module which is not the latest version.
	FetchAndUpdateModules(Vec<ModuleId>),
	UpdateModules(HashMap<ModuleId, /*install vs uninstall*/ bool>),
	// Poll what the latest version is for this specific source file.
	// If there is an update, download the updates.
	UpdateFile(SourceId),
}

#[derive(Clone, Derivative)]
#[derivative(PartialEq)]
pub struct Status {
	#[derivative(PartialEq="ignore")]
	rw_internal: Rc<RefCell<StatusState>>,
	r_external: UseStateHandle<StatusState>,
}
impl std::fmt::Debug for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("Status")
				.field("State", &self.rw_internal)
				.field("Display", &self.r_external)
				.finish()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
struct StatusState {
	stages: Vec<Stage>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Stage {
	pub title: AttrValue,
	pub progress: Option<Progress>,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Progress {
	pub max: usize,
	pub progress: usize,
}

impl Status {
	fn mutate(&self, perform: impl FnOnce(&mut StatusState)) {
		let mut state = self.rw_internal.borrow_mut();
		perform(&mut *state);
		self.r_external.set(state.clone());
	}

	pub fn push_stage(&self, title: impl Into<AttrValue>, max_progress: Option<usize>) {
		self.mutate(move |state| {
			state.stages.push(Stage {
				title: title.into(),
				progress: max_progress.map(|max| Progress { max, progress: 0 }),
			});
		});
	}

	pub fn pop_stage(&self) {
		self.mutate(move |state| {
			state.stages.pop();
		});
	}

	pub fn set_progress_max(&self, max: usize) {
		self.mutate(move |state| {
			let Some(stage) = state.stages.last_mut() else {
				log::error!(target: "autosync", "status has no stages");
				return;
			};
			let Some(progress) = &mut stage.progress else {
				log::error!(target: "autosync", "{stage:?} has no progress");
				return;
			};
			progress.max = max;
		});
	}

	pub fn increment_progress(&self) {
		self.mutate(move |state| {
			let Some(stage) = state.stages.last_mut() else {
				log::error!(target: "autosync", "status has no stages");
				return;
			};
			let Some(progress) = &mut stage.progress else {
				log::error!(target: "autosync", "{stage:?} has no progress");
				return;
			};
			progress.progress = progress.max.min(progress.progress + 1);
		});
	}

	pub fn is_active(&self) -> bool {
		!self.r_external.stages.is_empty()
	}

	pub fn stages(&self) -> &Vec<Stage> {
		&self.r_external.stages
	}
}

#[derive(thiserror::Error, Debug, Clone)]
enum StorageSyncError {
	#[error(transparent)]
	Database(#[from] crate::database::Error),
	#[error(transparent)]
	StorageError(#[from] super::github::Error),
}

#[function_component]
pub fn Provider(props: &ChildrenProps) -> Html {
	let database = use_context::<Database>().unwrap();
	let system_depot = use_context::<crate::system::Depot>().unwrap();
	let channel = Channel(use_memo((), |_| {
		let (send_req, recv_req) = async_channel::unbounded();
		RequestChannel { send_req, recv_req }
	}));
	let status = Status {
		rw_internal: Rc::new(RefCell::new(StatusState::default())),
		r_external: use_state_eq(|| StatusState {
			/*
			stages: vec![
				Stage {
					title: "Layer 1: Installing".into(),
					..Default::default()
				},
				Stage {
					title: "Layer 2: Modules".into(),
					progress: Some(Progress {
						progress: 2,
						max: 5,
					}),
				},
				Stage {
					title: "Layer 3: Files".into(),
					progress: Some(Progress {
						progress: 419,
						max: 650,
					}),
				}
			],
			// */
			..Default::default()
		}),
	};
	use_async_with_options(
		{
			let database = database.clone();
			let system_depot = system_depot.clone();
			let recv_req = channel.recv_req.clone();
			let status = status.clone();
			async move {
				while let Ok(req) = recv_req.recv().await {
					let auth_status = yewdux::dispatch::get::<crate::auth::Status>();
					let Some(storage) = auth_status.storage() else {
						log::error!(target: "autosync", "No storage available, cannot progess request {req:?}");
						continue;
					};

					let mut modules_to_update_or_fetch = BTreeMap::new();
					let mut modules_to_uninstall = BTreeMap::new();
					let mut scan_for_new_modules = false;
					let mut update_modules_out_of_date = false;
					match req {
						Request::FetchLatestVersionAllModules => {
							scan_for_new_modules = true;
							for module in database.clone().query_modules(None).await? {
								modules_to_update_or_fetch.insert(module.id.clone(), module);
							}
						}
						Request::UpdateModules(new_installation_status) => {
							update_modules_out_of_date = true;
							for (id, should_be_installed) in new_installation_status {
								let module = database.get::<Module>(id.to_string()).await?;
								if let Some(module) = module {
									if should_be_installed {
										modules_to_update_or_fetch.insert(module.id.clone(), module);
									} else {
										modules_to_uninstall.insert(module.id.clone(), module);
									}
								}
							}
						}
						Request::FetchAndUpdateModules(module_ids) => {
							update_modules_out_of_date = true;
							for id in module_ids {
								let module = database.get::<Module>(id.to_string()).await?;
								if let Some(module) = module {
									modules_to_update_or_fetch.insert(module.id.clone(), module);
								}
							}
						}
						Request::UpdateFile(source_id) => {
							update_modules_out_of_date = true;
							if let Some(id) = source_id.module {
								let module = database.get::<Module>(id.to_string()).await?;
								if let Some(module) = module {
									modules_to_update_or_fetch.insert(module.id.clone(), module);
								}
							}
						}
					}

					// TODO: Ensure that homebrew is always installed

					let repositories = if scan_for_new_modules {
						let mut query_module_owners = QueryModuleOwners {
							status: status.clone(),
							client: storage.clone(),
							found_homebrew: false,
						};
						let owners = query_module_owners.run().await?;

						// If the homebrew repo was not found when querying who the user is,
						// then we need to generate one, since this is where their user data is stored
						// and is the default location for any creations.
						if !query_module_owners.found_homebrew {
							let generate_homebrew = GenerateHomebrew {
								status: status.clone(),
								client: storage.clone(),
							};
							generate_homebrew.run().await?;
						}

						let scan_for_modules = ScanForModules {
							status: status.clone(),
							client: storage.clone(),
							owners,
						};
						scan_for_modules.run().await?
					} else {
						let module_names = modules_to_update_or_fetch.keys().map(ModuleId::to_string).collect();
						let find_modules = FindModules {
							status: status.clone(),
							client: storage.clone(),
							names: module_names,
						};
						find_modules.run().await?
					};

					status.push_stage("Updating database", None);
					commit_module_versions(&database, &repositories, &mut modules_to_update_or_fetch).await?;
					status.pop_stage();

					if !modules_to_uninstall.is_empty() {
						let transaction = database.write()?;
						for (id, mut module) in modules_to_uninstall {
							uninstall_module(&transaction, &mut module).await?;
						}
						transaction.commit().await.map_err(database::Error::from)?;
					}

					if update_modules_out_of_date {
						// scan modules for new content and download
						let module_names = modules_to_update_or_fetch.keys().map(ModuleId::to_string).collect::<Vec<_>>();

						struct ModuleUpdate {
							module: Module,
							files: Vec<ModuleFileUpdate>,
						}

						status.push_stage("Installing modules", None);

						let mut modules = Vec::with_capacity(modules_to_update_or_fetch.len());
						status.push_stage("Gathering updates", Some(modules_to_update_or_fetch.len()));
						for (_id, mut module) in modules_to_update_or_fetch {
							status.increment_progress();

							let ModuleId::Github { user_org, repository } = &module.id else {
								// ERROR: Invalid module id to scan
								continue;
							};
							
							// For prev uninstalled modules, scan the remote for all files at the latest state.
							if !module.installed {
								module.installed = true;

								let scan = ScanRepository {
									status: status.clone(),
									client: storage.clone(),
									owner: user_org.clone(),
									name: repository.clone(),
									tree_id: None,
								};
								let files = scan.run().await?;
								let files = files
									.into_iter()
									.map(|file| ModuleFileUpdate {
										file,
										status: ChangedFileStatus::Added,
									})
									.collect();

								modules.push(ModuleUpdate { module, files });
							}
							// For module updates, ask repo for changed files since current version.
							else if module.version != module.remote_version {
								let scan = FindFileUpdates {
									status: status.clone(),
									client: storage.clone(),
									owner: user_org.clone(),
									name: repository.clone(),
									old_version: module.version.clone(),
									new_version: module.remote_version.clone(),
								};
								module.version = module.remote_version.clone();

								let files = scan.run().await?;
								modules.push(ModuleUpdate { module, files });
							}
						}
						status.pop_stage(); // Gathering Updates

						// For all files to fetch, across all modules, fetch each file and update progress.
						// Iterate per module so updates can be committed to database as each is fetched.
						status.push_stage("Downloading Modules", Some(modules.len()));
						for ModuleUpdate { module, files } in modules {
							use crate::database::{
								app::{Entry, Module},
								ObjectStoreExt, TransactionExt,
							};
							
							status.increment_progress();

							let download = DownloadFileUpdates {
								status: status.clone(),
								client: storage.clone(),
								system_depot: system_depot.clone(),
								module_id: module.id.clone(),
								version: module.remote_version.clone(),
								files,
							};
							let (entries, removed_file_ids) = download.run().await?;

							status.push_stage(format!("Installing {}", module.id.to_string()), None);

							let transaction = database.write()?;

							let module_store = transaction.object_store_of::<Module>()?;
							module_store.put_record(&module).await?;

							let entry_store = transaction.object_store_of::<Entry>()?;
							for record in entries {
								entry_store.put_record(&record).await?;
							}
							// Delete entries by module and file-id
							let entry_ids_to_remove = {
								use futures_util::StreamExt;
								let mut cursor = Database::query_entries_in(&entry_store, &module.id).await?;
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

							transaction
								.commit()
								.await
								.map_err(|err| StorageSyncError::Database(err.into()))?;

							status.pop_stage(); // Installing owner/repo
						}
						status.pop_stage(); // Downloading Files
						status.pop_stage(); // Installing Modules
					}
				}
				Ok(()) as Result<(), StorageSyncError>
			}
		},
		UseAsyncOptions::enable_auto(),
	);

	html! {
		<ContextProvider<Channel> context={channel}>
			<ContextProvider<Status> context={status}>
				{props.children.clone()}
			</ContextProvider<Status>>
		</ContextProvider<Channel>>
	}
}

pub struct ModuleFile {
	// The game system the file is in.
	pub system: String,
	// The path within the module of the file (including game system root).
	pub path_in_repo: String,
	// The file-id sha in the github repo.
	pub file_id: String,
}
pub struct ModuleFileUpdate {
	pub file: ModuleFile,
	pub status: super::github::ChangedFileStatus,
}

impl ModuleFile {
	pub fn get_system_in_file_path(path: &std::path::Path) -> Option<String> {
		let Some(system_path) = path.components().next() else { return None; };
		let system = system_path.as_os_str().to_str().unwrap().to_owned();
		Some(system)
	}
}

// Commits a transaction to the database containing updates to local module versions and adding new uninstalled modules.
async fn commit_module_versions(
	database: &Database,
	remote_modules: &Vec<RepositoryMetadata>,
	local_modules: &mut BTreeMap<ModuleId, Module>,
) -> Result<(), StorageSyncError> {
	use crate::database::{ObjectStoreExt, TransactionExt};

	let transaction = database.write()?;
	let module_store = transaction.object_store_of::<Module>()?;

	for remote_repo in remote_modules {
		let remote_module_id = remote_repo.module_id();
		let module = match local_modules.remove(&remote_module_id) {
			Some(mut module) => {
				module.remote_version = remote_repo.version.clone();
				module
			}
			None => {
				let module = Module {
					id: remote_module_id.clone(),
					name: remote_module_id.to_string(),
					systems: remote_repo.systems.iter().cloned().collect(),
					version: remote_repo.version.clone(),
					remote_version: remote_repo.version.clone(),
					installed: false,
				};
				module
			}
		};
		module_store.put_record(&module).await?;
		local_modules.insert(remote_module_id, module);
	}

	transaction.commit().await.map_err(database::Error::from)?;

	Ok(())
}

async fn uninstall_module(transaction: &idb::Transaction, module: &mut Module) -> Result<(), database::Error> {
	use crate::database::{
		app::{entry::ModuleSystem, Entry},
		ObjectStoreExt, TransactionExt,
	};
	use futures_util::StreamExt;

	let module_store = transaction.object_store_of::<Module>()?;
	module.installed = false;
	module_store.put_record(module).await?;

	let entry_store = transaction.object_store_of::<Entry>()?;
	let idx_module_system = entry_store.index_of::<ModuleSystem>()?;
	for system in &module.systems {
		let query = ModuleSystem {
			module: module.id.to_string(),
			system: system.clone(),
		};
		let mut cursor = idx_module_system.open_cursor(Some(&query)).await?;
		while let Some(entry) = cursor.next().await {
			entry_store.delete_record(entry.id).await?;
		}
	}

	Ok(())
}
