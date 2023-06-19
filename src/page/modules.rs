use crate::{
	auth::{self, OAuthProvider},
	database::app::Database,
	storage::github::GithubClient,
	system, task,
};
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

	html! {<div class="m-2">
		<div class="d-flex justify-content-center">
			<button class="btn btn-outline-success me-2" onclick={initiate_loader}>{"Scan Github"}</button>
			<button class="btn btn-outline-danger me-2" onclick={clear_database}>{"Clear Downloaded Data"}</button>
		</div>

		<TaskListView />
	</div>}
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
