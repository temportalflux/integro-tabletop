use std::rc::Rc;

use super::app;
use crate::{
	components::Spinner,
	database::app::Database,
	system::{
		core::{ModuleId, SourceId},
		dnd5e::{
			data::character::{Persistent, PersistentMetadata},
			DnD5e,
		},
	},
};
use itertools::Itertools;
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncOptions};
use yew_router::{prelude::Link, Routable};

pub mod sheet;
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
				html!(<Sheet value={id} />)
			}
		}
	}
}

#[function_component]
pub fn CharacterLanding() -> Html {
	html! {<div>
		<h3 class="text-center">{"Characters"}</h3>
		<div class="d-flex align-items-center justify-content-center mb-1">
			<button
				class="btn btn-outline-success btn-sm ms-2"
				onclick={Callback::from({
					move |_| {}
				})}
			>
				{"New Character"}
			</button>
		</div>
		<CharacterList />
	</div>}
}

#[function_component]
pub fn CharacterList() -> Html {
	use crate::{
		components::database::{use_query_all, QueryAllArgs, QueryStatus},
		kdl_ext::KDLNode,
		system::{
			core::System,
			dnd5e::{data::character::Persistent, DnD5e},
		},
	};
	let character_entries = use_query_all(
		Persistent::id(),
		true,
		Some(QueryAllArgs {
			system: DnD5e::id().to_owned(),
			..Default::default()
		}),
	);

	match character_entries.status() {
		QueryStatus::Pending => html!(<Spinner />),
		QueryStatus::Empty | QueryStatus::Failed(_) => html!("No characters"),
		QueryStatus::Success(entries) => {
			let mut cards = Vec::with_capacity(entries.len());
			for entry in entries {
				let Ok(metadata) = serde_json::from_value::<PersistentMetadata>(entry.metadata.clone()) else { continue; };
				let id = entry.source_id(false);
				let route = Route::sheet(&id);
				cards.push(html!(<CharacterCard {id} {route} metadata={Rc::new(metadata)} />));
			}
			html! {
				<div class="d-flex align-items-center justify-content-center">
					{cards}
				</div>
			}
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
struct CharacterCardProps {
	id: SourceId,
	route: Route,
	metadata: Rc<PersistentMetadata>,
}
#[function_component]
fn CharacterCard(CharacterCardProps { id, route, metadata }: &CharacterCardProps) -> Html {
	html! {
		<div class="card m-1" style="min-width: 300px;">
			<div class="card-header d-flex align-items-center">
				<span>{&metadata.name}</span>
				<i
					class="bi bi-trash ms-auto"
					style="color: var(--bs-danger);"
					onclick={Callback::from({
						let id = id.clone();
						move |_| {
							log::debug!("TODO: open delete modal for {:?}", id.to_string());
						}
					})}
				/>
			</div>
			<div class="card-body">
				{match metadata.pronouns.is_empty() {
					true => html!(),
					false => html!(<div>{metadata.pronouns.join(", ")}</div>),
				}}
				<div>
					{"Level "}{metadata.level}{": "}
					{metadata.classes.iter().map(|(class_name, subclass_name)| html! {
						<span>
							{class_name}
							{subclass_name.as_ref().map(|name| {
								html!(<>{"/"}{name}</>)
							}).unwrap_or_default()}
						</span>
					}).collect::<Vec<_>>()}
				</div>
				<div>
					{metadata.bundles.iter_all()
						.filter(|(category, _)| {
							vec!["Race", "RaceVariant", "Lineage", "Upbringing"].contains(&category.as_str())
						})
						.sorted_by_key(|(category, _)| *category)
						.map(|(_category, items)| items.clone())
						.flatten()
						.collect::<Vec<_>>()
						.join(", ")
					}
				</div>
				<div class="d-flex justify-content-center mt-2">
					<Link<Route>
						to={route.clone()}
						classes="btn btn-success btn-sm me-2"
					>
						{"Open"}
					</Link<Route>>
				</div>
			</div>
		</div>
	}
}
