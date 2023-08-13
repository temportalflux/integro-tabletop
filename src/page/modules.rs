use crate::{
	auth::{self, OAuthProvider},
	components::{
		database::{use_query_modules, QueryStatus},
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

	let initiate_loader = Callback::from({
		let database = database.clone();
		let task_dispatch = task_dispatch.clone();
		move |_| {
			let auth::Status::Successful { provider, token } = &*auth_status else { return; };
			if *provider != OAuthProvider::Github {
				log::error!("Currently authenticated with {provider:?}, but no storage system is hooked up for anything but github.");
				return;
			}
			let Ok(client) = GithubClient::new(token) else {
				return;
			};
			loader::Loader {
				client,
				task_dispatch: task_dispatch.clone(),
				system_depot: system_depot.clone(),
				database: database.clone(),
			}
			.find_and_download_modules();
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

			<ModuleList />
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
fn ModuleList() -> Html {
	let modules_query = use_query_modules(None);
	match modules_query.status() {
		QueryStatus::Pending => html!(<Spinner />),
		QueryStatus::Empty | QueryStatus::Failed(_) => html! {
			{"You have no modules on this device. Scanning storage or creating a module!"}
		},
		QueryStatus::Success(modules) => {
			let total_module_count = modules.len();
			let mut by_system = BTreeMap::<&String, Vec<Module>>::new();
			for module in modules {
				match by_system.get_mut(&module.system) {
					None => {
						by_system.insert(&module.system, vec![module.clone()]);
					}
					Some(group) => {
						let idx = group.binary_search_by(|probe| probe.name.cmp(&module.name));
						let idx = idx.unwrap_or_else(|err_idx| err_idx);
						group.insert(idx, module.clone());
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
							{modules.into_iter().map(|module| html!(<ModuleCard value={module} />)).collect::<Vec<_>>()}
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

#[function_component]
fn ModuleCard(GeneralProp { value }: &GeneralProp<Module>) -> Html {
	html! {
		<div class="card m-1" style="min-width: 300px;">
			<div class="card-header">
				<span>{&value.name}</span>
			</div>
			<div class="card-body">
				<div>
					{"Version: "}
					{match value.version.get(0..8) {
						None => html!("Unknown"),
						Some(ver) => html!({ver}),
					}}
				</div>
				<div>
					{"System: "}
					{&value.system}
				</div>
			</div>
		</div>
	}
}
