use super::app;
use crate::{
	database::app::Database,
	system::core::{ModuleId, SourceId},
};
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncOptions};
use yew_router::{prelude::Link, Routable};

mod sheet;
use sheet::Sheet;

#[function_component]
pub fn Switch() -> Html {
	html!(<yew_router::Switch<Route> render={Route::switch} />)
}

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
	#[at("/characters")]
	Landing,
	#[at("/characters/:storage/:module/:system/*path")]
	Sheet {
		storage: String,
		module: String,
		system: String,
		path: String,
	},
	#[not_found]
	#[at("/characters/404")]
	NotFound,
}

impl Route {
	/// Returns the character sheet route for a character identified by the source id.
	/// If the id doesn't have a module & system, or otherwise could not be parsed, the NotFound route is returned.
	pub fn sheet(id: &SourceId) -> Self {
		let Some(module_id) = &id.module else { return Self::NotFound; };
		let Some(system) = &id.system else { return Self::NotFound; };
		let (storage, module) = match module_id {
			ModuleId::Local { name } => ("local", name.clone()),
			ModuleId::Github {
				user_org,
				repository,
			} => ("github", format!("{user_org}:{repository}")),
		};
		Self::Sheet {
			storage: storage.to_owned(),
			module,
			system: system.clone(),
			path: id.path.display().to_string(),
		}
	}

	fn switch(self) -> Html {
		match self {
			Self::Landing => html!(<CharacterLanding />),
			Self::NotFound => app::Route::not_found(),
			Self::Sheet {
				storage,
				module,
				system,
				path,
			} => {
				let module = match storage.as_str() {
					"local" => ModuleId::Local { name: module },
					"github" => {
						let mut parts = module.split(':');
						let Some(user_org) = parts.next() else {
							return app::Route::not_found();
						};
						let Some(repo) = parts.next() else {
							return app::Route::not_found();
						};
						ModuleId::Github {
							user_org: user_org.to_owned(),
							repository: repo.to_owned(),
						}
					}
					_ => return app::Route::not_found(),
				};
				let id = SourceId {
					module: Some(module),
					system: Some(system),
					path: std::path::PathBuf::from(path),
					version: None,
					..Default::default()
				};
				log::debug!("{}", id.to_string());
				html!(<Sheet />)
			}
		}
	}
}

#[function_component]
pub fn CharacterLanding() -> Html {
	html! {<>
		<CharacterList />
	</>}
}

#[function_component]
pub fn CharacterList() -> Html {
	let database = use_context::<Database>().unwrap();
	let character_entries = use_async_with_options(
		{
			let database = database.clone();
			async move {
				use crate::{
					database::{app::Entry, Error},
					kdl_ext::KDLNode,
					system::{
						core::System,
						dnd5e::{data::character::Persistent, DnD5e},
					},
				};
				use futures_util::StreamExt;
				let mut entries = Vec::new();
				let mut stream = database
					.query_entries(DnD5e::id(), Persistent::id(), None)
					.await?;
				while let Some(entry) = stream.next().await {
					entries.push(entry);
				}
				Ok(entries) as Result<Vec<Entry>, Error>
			}
		},
		UseAsyncOptions::enable_auto(),
	);
	let content = if character_entries.loading {
		html! {
			<div class="spinner-border" role="status">
				<span class="visually-hidden">{"Loading..."}</span>
			</div>
		}
	} else if let Some(entries) = &character_entries.data {
		html! {<>
			{entries.iter().map(|entry| {
				let name = entry.get_meta_str("name").unwrap_or("No Name");
				let route = Route::sheet(&entry.source_id(false));
				html! {
					<div class="d-flex align-items-center mb-2">
						<Link<Route>
							to={route}
							classes="btn btn-success btn-sm me-2"
						>
							{"Open"}
						</Link<Route>>
						{name}
					</div>
				}
			}).collect::<Vec<_>>()}
		</>}
	} else {
		html!("No characters")
	};
	html! {
		<div>
			<div class="d-flex align-items-center mb-1">
				<h3>{"Characters"}</h3>
				<button
					class="btn btn-outline-secondary btn-sm ms-2"
					onclick={Callback::from({
						let task = character_entries.clone();
						move |_| task.run()
					})}
				>
					{"Refresh"}
				</button>
			</div>
			{content}
		</div>
	}
}
