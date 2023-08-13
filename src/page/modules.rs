use crate::{
	auth::{self, OAuthProvider},
	components::{
		database::{use_query_modules, QueryStatus, UseQueryModulesHandle},
		Spinner,
	},
	database::app::{Database, Module},
	storage::github::GithubClient,
	system::{self, dnd5e::components::GeneralProp},
	task,
};
use std::collections::BTreeMap;
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
	let modules_query = use_query_modules(None);

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
				<button class="btn btn-outline-success me-2" onclick={initiate_loader}>{"Scan Github"}</button>
				<button class="btn btn-outline-danger me-2" onclick={clear_database}>{"Clear Downloaded Data"}</button>
			</div>

			<TaskListView />

			<ModuleList value={modules_query.clone()} />
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

#[function_component]
fn ModuleList(
	GeneralProp {
		value: modules_query,
	}: &GeneralProp<UseQueryModulesHandle>,
) -> Html {
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
								<ModuleCard {module} on_delete={on_delete_module.clone()} />
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
}
#[function_component]
fn ModuleCard(ModuleCardProps { module, on_delete }: &ModuleCardProps) -> Html {
	let database = use_context::<Database>().unwrap();
	let task_dispatch = use_context::<task::Dispatch>().unwrap();
	let on_delete = Callback::from({
		let module_id = module.module_id.clone();
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
					Error,
					app::{Entry, Module, entry::ModuleSystem},
					ObjectStoreExt, TransactionExt,
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
			</div>
		</div>
	}
}
