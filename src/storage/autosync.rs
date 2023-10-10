use crate::{
	database::app::{Database, Module},
	storage::github::RepositoryMetadata,
	system::core::{ModuleId, SourceId},
};
use std::{collections::{BTreeMap, HashMap}, rc::Rc};
use yew::{html::ChildrenProps, prelude::*};
use yew_hooks::*;

mod query_module_owners;
use query_module_owners::*;
mod generate_homebrew;
use generate_homebrew::*;
mod scan_for_modules;
use scan_for_modules::*;
mod find_modules;
use find_modules::*;

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

#[derive(Clone, PartialEq)]
pub struct Status(UseStateHandle<StatusState>);

#[derive(Clone, PartialEq, Default)]
struct StatusState {
	is_active: bool,
	title: Option<AttrValue>,
	progress: Option<(usize, usize)>,
	progress_description: Option<AttrValue>,
}

impl Status {
	fn mutate(&self, perform: impl FnOnce(&mut StatusState)) {
		let mut update = (*self.0).clone();
		perform(&mut update);
		self.0.set(update);
	}

	pub fn deactivate(&self) {
		self.mutate(move |state| {
			state.is_active = false;
			state.title = None;
		});
	}

	pub fn activate_with_title(&self, title: impl Into<AttrValue>, progress_max: Option<usize>) {
		self.mutate(move |state| {
			state.title = Some(title.into());
			state.is_active = true;
			state.progress = progress_max.map(|max| (0, max));
		});
	}

	pub fn set_progress_description(&self, description: impl Into<AttrValue>) {
		self.mutate(move |state| {
			state.progress_description = Some(description.into());
		});
	}

	pub fn increment_progress(&self) {
		self.mutate(move |state| {
			if let Some((progress, max)) = &mut state.progress {
				*progress = (*max).min(*progress + 1);
			}
		});
	}

	pub fn is_active(&self) -> bool {
		self.0.is_active
	}

	pub fn title(&self) -> Option<&AttrValue> {
		self.0.title.as_ref()
	}

	pub fn progress(&self) -> Option<(usize, usize)> {
		self.0.progress
	}

	pub fn progress_description(&self) -> Option<&AttrValue> {
		self.0.progress_description.as_ref()
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
	let status = Status(use_state_eq(|| StatusState::default()));
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
									}
									else {
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

					let repositories = if scan_for_new_modules {
						log::debug!(target: "autosync", "scanning storage for modules");

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
							log::debug!(target: "autosync", "generating homebrew");
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

					status.activate_with_title("Updating database", None);
					commit_module_versions(&database, &repositories, &mut modules_to_update_or_fetch).await?;
					status.deactivate();

					// TODO: uninstall desired modules (mark as uninstalled, and delete all content entries for that module)
					if !modules_to_uninstall.is_empty() {
						let module_names = modules_to_uninstall.keys().map(ModuleId::to_string).collect::<Vec<_>>();
						log::debug!(target: "autosync", "uninstall modules {module_names:?}");
					}

					if update_modules_out_of_date {
						// scan modules for new content and download
						log::debug!(target: "autosync", "update out of date modules");
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

// Commits a transaction to the database containing updates to local module versions and adding new uninstalled modules.
async fn commit_module_versions(
	database: &Database,
	remote_modules: &Vec<RepositoryMetadata>,
	local_modules: &mut BTreeMap<ModuleId, Module>,
) -> Result<(), StorageSyncError> {
	use crate::database::{self, ObjectStoreExt, TransactionExt};

	let transaction = database.write()?;
	let module_store = transaction.object_store_of::<Module>()?;
	
	for remote_repo in remote_modules {
		let remote_module_id = remote_repo.module_id();
		let module = match local_modules.remove(&remote_module_id) {
			Some(mut module) => {
				module.remote_version = remote_repo.version.clone();
				log::debug!(
					target: "autosync",
					"Updating remote version of {:?} to {:?}",
					remote_module_id.to_string(), remote_repo.version
				);
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
				log::debug!(
					target: "autosync",
					"Inserting new uninstalled module {:?} @ {:?}",
					remote_module_id.to_string(), remote_repo.version
				);
				module
			}
		};
		module_store.put_record(&module).await?;
		local_modules.insert(remote_module_id, module);
	}

	transaction.commit().await.map_err(database::Error::from)?;

	Ok(())
}
