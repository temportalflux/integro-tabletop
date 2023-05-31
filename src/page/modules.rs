use crate::{auth, database::app::Database, storage::github::GithubClient, system, task};
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
		move |_| {
			let auth::Status::Successful { token } = &*auth_status else { return; };
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

	html! {<>
		<button class="btn btn-outline-success" onclick={initiate_loader}>{"Scan Github"}</button>
		<TaskListView />
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
