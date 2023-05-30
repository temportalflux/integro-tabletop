use super::app;
use crate::system::core::{ModuleId, SourceId};
use yew::prelude::*;
use yew_router::{Routable, prelude::Link};

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
	use std::str::FromStr;
	let karl_id = SourceId::from_str("github://temportalflux:integro-homebrew@dnd5e/characters/clericKarl.kdl").unwrap();
	html! {<>
		{"You can view all your characters here"}
		<Link<Route> to={Route::sheet(&karl_id)}>{"Karl the Cleric"}</Link<Route>>
	</>}
}
