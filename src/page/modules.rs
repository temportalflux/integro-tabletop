use crate::{
	auth::{self, OAuthProvider},
	components::{
		database::{use_query_modules, QueryStatus, UseQueryModulesHandle},
		Spinner, stop_propagation,
	},
	database::app::{Database, Module},
	storage::{github::GithubClient, autosync},
	system::{self, dnd5e::components::GeneralProp, core::ModuleId},
	task, utility::InputExt,
};
use std::collections::{BTreeMap, HashSet, HashMap};
use yew::prelude::*;
use yewdux::prelude::*;

mod loader;

/// Page which displays the modules the user currently logged in has contributor access to.
#[function_component]
pub fn ModulesLanding() -> Html {
	let database = use_context::<Database>().unwrap();
	let task_dispatch = use_context::<task::Dispatch>().unwrap();
	let system_depot = use_context::<system::Depot>().unwrap();
	let (auth_status, _) = use_store::<auth::Status>();
	let autosync_channel = use_context::<autosync::Channel>().unwrap();
	let modules_query = use_query_modules(None);

	let pending_module_installations = use_state_eq(|| HashMap::<ModuleId, bool>::new());

	let initiate_loader = Callback::from({
		let database = database.clone();
		let task_dispatch = task_dispatch.clone();
		let modules_query = modules_query.clone();
		move |_| {
			let auth::Status::Successful { provider, token } = &*auth_status else { return; };
			if *provider != OAuthProvider::Github {
				log::error!("Currently authenticated with {provider:?}, but no storage system is hooked up for anything but github.");
				return;
			}
			let Ok(client) = GithubClient::new(token) else {
				return;
			};
			let on_finished = Box::new({
				let modules_query = modules_query.clone();
				move || {
					modules_query.run();
				}
			});
			let loader = loader::Loader {
				client,
				task_dispatch: task_dispatch.clone(),
				system_depot: system_depot.clone(),
				database: database.clone(),
				on_finished,
			};
			loader.find_and_download_modules();
		}
	});
	let clear_database = Callback::from({
		let database = database.clone();
		let task_dispatch = task_dispatch.clone();
		move |_| {
			let database = database.clone();
			task_dispatch.spawn("Clear Database", None, async move {
				database.clear().await?;
				Ok(()) as Result<(), crate::database::Error>
			});
		}
	});

	html! {<>
		<crate::components::modal::GeneralPurpose />
		<div class="m-2">
			<div class="d-flex justify-content-center">
				<button
					class="btn btn-outline-success me-2"
					onclick={Callback::from({
						let channel = autosync_channel.clone();
						move |_| {
							channel.try_send_req(autosync::Request::FetchLatestVersionAllModules);
						}
					})}
				>{"Scan Storage"}</button>
				<button
					class="btn btn-outline-success me-2"
					disabled={pending_module_installations.is_empty()}
					onclick={Callback::from({
						let channel = autosync_channel.clone();
						let pending_changes = pending_module_installations.clone();
						move |_| {
							let changes = (*pending_changes).clone();
							pending_changes.set(HashMap::new());
							channel.try_send_req(autosync::Request::UpdateModules(changes));
						}
					})}
				>{"Installed Selected"}</button>
				<button class="btn btn-outline-success me-2" onclick={initiate_loader}>{"Initiate Old Loader"}</button>
				<button class="btn btn-outline-danger me-2" onclick={clear_database}>{"Clear Downloaded Data"}</button>
			</div>

			<TaskListView />

			<ModuleList modules_query={modules_query.clone()} {pending_module_installations} />
		</div>
	</>}
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

#[derive(Clone, PartialEq, Properties)]
struct ModuleListProps {
	modules_query: UseQueryModulesHandle,
	pending_module_installations: UseStateHandle<HashMap<ModuleId, bool>>,
}

#[function_component]
fn ModuleList(ModuleListProps { modules_query, pending_module_installations }: &ModuleListProps) -> Html {
	let on_delete_module = Callback::from({
		let modules_query = modules_query.clone();
		move |_| {
			modules_query.run();
		}
	});
	match modules_query.status() {
		QueryStatus::Pending => html!(<Spinner />),
		QueryStatus::Empty | QueryStatus::Failed(_) => html! {
			{"You have no modules on this device. Scanning storage or creating a module!"}
		},
		QueryStatus::Success(modules) => {
			let total_module_count = modules.len();
			let mut by_system = BTreeMap::<&String, Vec<Module>>::new();
			for module in modules {
				for system in &module.systems {
					match by_system.get_mut(system) {
						None => {
							by_system.insert(system, vec![module.clone()]);
						}
						Some(group) => {
							let idx = group.binary_search_by(|probe| probe.name.cmp(&module.name));
							let idx = idx.unwrap_or_else(|err_idx| err_idx);
							group.insert(idx, module.clone());
						}
					}
				}
			}
			let sys_count = by_system.len();
			let mut sections_by_system = Vec::with_capacity(sys_count);
			for (system_id, modules) in by_system {
				sections_by_system.push(html! {
					<div>
						<h4>{system_id}</h4>
						<div class="d-flex flex-wrap">
							{modules.into_iter().map(|module| html! {
								<ModuleCard
									{module} on_delete={on_delete_module.clone()}
									pending_module_installations={pending_module_installations.clone()}
								/>
							}).collect::<Vec<_>>()}
						</div>
					</div>
				});
			}
			html! {
				<div>
					{format!("These are {total_module_count} modules downloaded to your device across {sys_count} game systems.")}
					{sections_by_system}
				</div>
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
struct ModuleCardProps {
	module: Module,
	on_delete: Callback<()>,
	pending_module_installations: UseStateHandle<HashMap<ModuleId, bool>>,
}
#[function_component]
fn ModuleCard(props: &ModuleCardProps) -> Html {
	let ModuleCardProps { module, on_delete, pending_module_installations } = props;
	let database = use_context::<Database>().unwrap();
	let task_dispatch = use_context::<task::Dispatch>().unwrap();
	let on_delete = Callback::from({
		let module_id = module.id.clone();
		let task_dispatch = task_dispatch.clone();
		let database = database.clone();
		let on_delete = on_delete.clone();
		move |_| {
			// TODO: Modal which checks if any characters depend on the module,
			// and only allows deletion if there are no dependees.

			// TODO: Next - delete the module at the provided id & delete all records which are associated with that module in any system. Can use a database index for this, similar to modules_system.
			let module_id = module_id.clone();
			let database = database.clone();
			let on_delete = on_delete.clone();
			task_dispatch.spawn(format!("Delete {}", module_id.to_string()), None, async move {
				use crate::database::{
					app::{entry::ModuleSystem, Entry, Module},
					Error, ObjectStoreExt, TransactionExt,
				};
				use futures_util::StreamExt;
				let transaction = database.write()?;
				let module_store = transaction.object_store_of::<Module>()?;
				let entry_store = transaction.object_store_of::<Entry>()?;

				let module_systems = {
					let req = module_store.get_record::<Module>(module_id.to_string());
					let module = req.await?.unwrap();
					module.systems
				};

				let mut entry_ids = Vec::new();
				let idx_module_system = entry_store.index_of::<ModuleSystem>()?;
				for system in module_systems {
					let query = ModuleSystem {
						module: module_id.to_string(),
						system,
					};
					let mut cursor = idx_module_system.open_cursor(Some(&query)).await?;
					while let Some(entry) = cursor.next().await {
						entry_ids.push(entry.id);
					}
				}

				for entry_id in entry_ids {
					entry_store.delete_record(entry_id).await?;
				}
				module_store.delete_record(module_id.to_string()).await?;

				transaction.commit().await?;
				on_delete.emit(());
				Ok(()) as Result<(), Error>
			});
		}
	});
	let show_as_installed = pending_module_installations.get(&module.id).copied().unwrap_or(module.installed);
	html! {
		<div class="card m-1" style="min-width: 300px;">
			<div class="card-header d-flex align-items-center">
				<span>{&module.name}</span>
				<i
					class="bi bi-trash ms-auto"
					style="color: var(--bs-danger);"
					onclick={on_delete}
				/>
			</div>
			<div class="card-body">
				<div>
					{"Version: "}
					{match module.version.get(0..8) {
						None => html!("Unknown"),
						Some(ver) => html!({ver}),
					}}
				</div>
				<div>
					{"Systems: "}
					{module.systems.iter().cloned().collect::<Vec<_>>().join(", ")}
				</div>
				<div>
					<input
						type="checkbox"
						class={classes!("form-check-input", "slot", "success")}
						checked={show_as_installed}
						onclick={stop_propagation()}
						onchange={Callback::from({
							let module_id = module.id.clone();
							let is_installed = module.installed;
							let pending_module_installations = pending_module_installations.clone();
							move |evt: web_sys::Event| {
								let Some(should_be_installed) = evt.input_checked() else { return; };
								let should_be_pending = should_be_installed != is_installed;
								let has_pending_entry = pending_module_installations.contains_key(&module_id);
								if should_be_pending != has_pending_entry {
									let mut pending_entries = (*pending_module_installations).clone();
									if has_pending_entry {
										pending_entries.remove(&module_id);
									}
									else {
										pending_entries.insert(module_id.clone(), should_be_installed);
									}
									pending_module_installations.set(pending_entries);
								}
							}
						})}
					/>
					{"Installed"}
				</div>
			</div>
		</div>
	}
}
