use super::app;
use crate::{
	components::{modal, Spinner},
	database::app::Database,
	system::{
		core::{ModuleId, SourceId, System},
		dnd5e::{
			components::GeneralProp,
			data::character::{Persistent, PersistentMetadata},
			DnD5e,
		},
	},
	utility::InputExt, task::Signal,
};
use itertools::Itertools;
use std::{path::Path, rc::Rc};
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncOptions};
use yew_router::{prelude::Link, Routable};
use yewdux::prelude::use_store;

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
	let modal_dispatcher = use_context::<modal::Context>().unwrap();

	let on_create = modal_dispatcher.callback({
		move |_| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<ModalCreate />},
				..Default::default()
			})
		}
	});
	let on_delete = modal_dispatcher.callback({
		move |id: SourceId| {
			modal::Action::Open(modal::Props {
				centered: true,
				scrollable: true,
				content: html! {<ModalDelete value={id} />},
				..Default::default()
			})
		}
	});

	html! {<>
		<crate::components::modal::GeneralPurpose />
		<div>
			<h3 class="text-center">{"Characters"}</h3>
			<div class="d-flex align-items-center justify-content-center mb-1">
				<button
					class="btn btn-outline-success btn-sm ms-2"
					onclick={on_create}
				>
					{"New Character"}
				</button>
			</div>
			<CharacterList value={on_delete} />
		</div>
	</>}
}

#[function_component]
pub fn CharacterList(GeneralProp { value: on_delete }: &GeneralProp<Callback<SourceId>>) -> Html {
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
				cards.push(html!(<CharacterCard
					{id} {route}
					metadata={Rc::new(metadata)}
					on_delete={on_delete.clone()}
				/>));
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
	on_delete: Callback<SourceId>,
}
#[function_component]
fn CharacterCard(
	CharacterCardProps {
		id,
		route,
		metadata,
		on_delete,
	}: &CharacterCardProps,
) -> Html {
	html! {
		<div class="card m-1" style="min-width: 300px;">
			<div class="card-header d-flex align-items-center">
				<span>{&metadata.name}</span>
				<i
					class="bi bi-trash ms-auto"
					style="color: var(--bs-danger);"
					onclick={on_delete.reform({
						let id = id.clone();
						move |_| id.clone()
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

#[function_component]
fn ModalCreate() -> Html {
	let (auth_status, _dispatch) = use_store::<crate::auth::Status>();
	let task_dispatch = use_context::<crate::task::Dispatch>().unwrap();

	// TODO: Select the parent module

	let filename = use_state_eq(|| "unnamed".to_owned());
	let on_change_filename = Callback::from({
		let filename = filename.clone();
		move |evt: web_sys::Event| {
			let Some(value) = evt.input_value() else { return; };
			filename.set(value);
		}
	});

	let creation_in_progress = use_state(|| Signal::new(false));
	let onclick = Callback::from({
		let auth_status = auth_status.clone();
		let task_dispatch = task_dispatch.clone();
		let filename = filename.clone();
		let creation_in_progress = creation_in_progress.clone();
		move |_| {
			if creation_in_progress.value() {
				return;
			}
			let Some(client) = auth_status.storage() else {
				log::debug!("no storage client");
				return;
			};
			let path_in_repo = Path::new(DnD5e::id())
				.join("character")
				.join(format!("{}.kdl", filename.as_str()));
			let signal = task_dispatch.spawn("Create Character File", None, async move {
				let (_, homebrew_repo) = client.viewer().await?;
				let Some(homebrew_repo) = homebrew_repo else { return Ok(()); };
				let message = "Add new character";
				let state = Persistent::default();
				let content = {
					let doc = state.export_as_kdl();
					let doc = doc.to_string();
					let doc = doc.replace("\\r", "");
					let doc = doc.replace("\\n", "\n");
					let doc = doc.replace("\\t", "\t");
					let doc = doc.replace("    ", "\t");
					doc
				};
				let args = crate::storage::github::CreateOrUpdateFileArgs {
					repo_org: &homebrew_repo.owner,
					repo_name: &homebrew_repo.name,
					path_in_repo: &path_in_repo,
					commit_message: &message,
					content: &content,
					file_id: None,
					branch: None,
				};
				log::debug!("{args:?}");
				client.create_or_update_file(args).await?;
				log::debug!("complete");
				Ok(()) as anyhow::Result<()>
			});
			creation_in_progress.set(signal);
		}
	});

	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"New Character"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			<div class="mb-3">
				<label class="form-label" for="filename">{"Unique File Name"}</label>
				<input
					class="form-control" id="filename" type="text"
					value={(*filename).clone()}
					onchange={on_change_filename}
				/>
				<div class="form-text">{"This is a unique id used to save the file."}</div>
			</div>
			<div class="d-flex justify-content-center">
				<button class="btn btn-success m-2" {onclick}>{"Create Character"}</button>
			</div>
		</div>
	</>}
}

#[function_component]
fn ModalDelete(GeneralProp { value: id }: &GeneralProp<SourceId>) -> Html {
	// TODO: This could use a confirmation captcha, requiring that the character's name by typed in.
	let onclick = Callback::from({
		let id = id.clone();
		move |_| {
			log::debug!("TODO: Delete {:?}", id.to_string());
		}
	});
	html! {<>
		<div class="modal-header">
			<h1 class="modal-title fs-4">{"Delete Character"}</h1>
			<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" />
		</div>
		<div class="modal-body">
			{"Are you sure you want to delete this character? You wont be able to undo this action \
			within this app (you need to know how to use git to do so)."}
			<div class="d-flex justify-content-center">
				<button class="btn btn-danger m-2" {onclick}>{"Delete Character"}</button>
			</div>
		</div>
	</>}
}
