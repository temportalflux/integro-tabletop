use std::sync::Arc;
use yew::prelude::*;

pub mod auth;
pub mod bootstrap;
pub mod components;
pub mod data;
pub mod database;
pub mod kdl_ext;
pub mod logging;
pub mod page;
pub mod path_map;
pub mod storage;
pub mod system;
pub mod task;
pub mod theme;
pub mod utility;

#[cfg(target_family = "wasm")]
fn main() {
	logging::wasm::init(logging::wasm::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[function_component]
fn App() -> Html {
	html! {<>
		<ProviderChain>
			<crate::components::modal::Provider>
				<crate::components::modal::GeneralPurpose />
				<page::App />
			</crate::components::modal::Provider>
		</ProviderChain>
	</>}
}

#[function_component]
fn ProviderChain(props: &html::ChildrenProps) -> Html {
	use crate::components::{mobile, object_browser};
	html! {
		<mobile::Provider threshold={1200}>
			<auth::ActionProvider>
				<task::Provider>
					<system::Provider>
						<DatabaseProvider>
							<object_browser::Provider>
								{props.children.clone()}
							</object_browser::Provider>
						</DatabaseProvider>
					</system::Provider>
				</task::Provider>
			</auth::ActionProvider>
		</mobile::Provider>
	}
}

#[function_component]
fn DatabaseProvider(props: &html::ChildrenProps) -> Html {
	use database::app::Database;
	let database = yew_hooks::use_async(async move {
		match Database::open().await {
			Ok(db) => Ok(db),
			Err(err) => {
				log::error!(target: "tabletop-tools", "Failed to connect to database: {err:?}");
				Err(Arc::new(err))
			}
		}
	});
	// When the app first opens, load the database.
	// Could probably check `use_is_first_mount()`, but checking if there database
	// doesn't exist yet and isn't loading is more clear.
	if database.data.is_none() && !database.loading {
		database.run();
	}
	// If the database has not yet loaded (or encountered an error),
	// we wont even show the children - mostly to avoid the numerous errors that would occur
	// since children strongly rely on the database existing.
	let Some(ddb) = &database.data else { return html!(); };
	html! {
		<ContextProvider<Database> context={ddb.clone()}>
			{props.children.clone()}
		</ContextProvider<Database>>
	}
}

#[cfg(target_family = "windows")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	use crate::system::{
		core::{ModuleId, SourceId},
		dnd5e,
	};
	use anyhow::Context;
	use std::{collections::BTreeMap, sync::Arc};

	let _ = logging::console::init("tabletop-tools", &[]);

	let comp_reg = dnd5e::component_registry();
	let node_reg = Arc::new(dnd5e::node_registry());

	let mut modules = Vec::new();
	for entry in std::fs::read_dir("./modules")? {
		let entry = entry?;
		if !entry.metadata()?.is_dir() {
			continue;
		}
		let module_id = entry.file_name().to_str().unwrap().to_owned();
		let mut system_ids = Vec::new();
		for entry in std::fs::read_dir(entry.path())? {
			let entry = entry?;
			if !entry.metadata()?.is_dir() {
				continue;
			}
			let system_id = entry.file_name().to_str().unwrap().to_owned();
			system_ids.push(system_id);
		}
		log::debug!("Found module {module_id:?} with systems {system_ids:?}.");
		modules.push((entry.path(), module_id, system_ids));
	}

	let mut sources = BTreeMap::new();
	for (module_path, module_id, system_ids) in modules {
		for system_id in system_ids {
			log::info!("Loading module \"{module_id}/{system_id}\"");
			let system_path = module_path.join(&system_id);
			let mut item_paths = Vec::new();
			for item in WalkDir::new(&system_path) {
				let Some(ext) = item.extension() else { continue; };
				if ext.to_str() != Some("kdl") {
					continue;
				}
				let Ok(content) = std::fs::read_to_string(&item) else { continue; };
				let item_relative_path = item.strip_prefix(&system_path)?;
				item_paths.push(item_relative_path.to_owned());
				let source_id = SourceId {
					module: Some(ModuleId::Local {
						name: module_id.clone(),
					}),
					system: Some(system_id.clone()),
					path: item_relative_path.to_owned(),
					..Default::default()
				};
				sources.insert(source_id, content);
			}
			// Update the index file
			tokio::fs::write(system_path.join("index"), {
				item_paths
					.into_iter()
					.map(|path| path.display().to_string().replace("\\", "/"))
					.collect::<Vec<_>>()
					.join("\n")
			})
			.await?;
		}
	}

	for (mut source_id, content) in sources {
		let document = content
			.parse::<kdl::KdlDocument>()
			.with_context(|| format!("Invalid KDL format in {:?}", source_id.to_string()))?;
		let mut reserialized_nodes = Vec::with_capacity(document.nodes().len());
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let node_name = node.name().value();
			let Some(comp_factory) = comp_reg.get_factory(node_name).cloned() else {
				log::error!("Failed to find factory to deserialize node \"{node_name}\".");
				continue;
			};
			let ctx = kdl_ext::NodeContext::new(Arc::new(source_id.clone()), node_reg.clone());
			#[allow(unused_variables)]
			let metadata = comp_factory.metadata_from_kdl(kdl_ext::NodeReader::new_root(node, ctx))?;
			/*
			if node_name == "bundle" {
				log::debug!("{}", metadata.to_string());
			}
			*/

			// NOTE: This will re-write the local data using the re-serialized node.
			// Do not enable unless you are specifically testing input vs output on documents.
			//reserialized_nodes.push(comp_factory.reserialize_kdl(kdl_ext::NodeReader::new_root(node, ctx))?);
		}
		if !reserialized_nodes.is_empty() {
			let Some(ModuleId::Local { name: module_name }) = &source_id.module else { continue; };
			let Some(system) = &source_id.system else { continue; };
			let dest_path = std::path::PathBuf::from(format!("./modules/{module_name}/{system}"))
				.join(&source_id.path);
			let mut doc = kdl::KdlDocument::new();
			doc.nodes_mut().append(&mut reserialized_nodes);
			let out_str = doc.to_string();
			let out_str = out_str.replace("\\r", "");
			let out_str = out_str.replace("\\n", "\n");
			let out_str = out_str.replace("\\t", "\t");
			let out_str = out_str.replace("    ", "\t");
			let _ = std::fs::write(dest_path, out_str);
		}
	}

	Ok(())
}

#[derive(thiserror::Error, Debug)]
pub struct GeneralError(pub String);
impl std::fmt::Display for GeneralError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

struct WalkDir {
	iter: Option<std::fs::ReadDir>,
	stack: Vec<std::fs::ReadDir>,
}
impl WalkDir {
	fn new(path: impl AsRef<std::path::Path>) -> Self {
		Self {
			iter: std::fs::read_dir(path).ok(),
			stack: Vec::new(),
		}
	}
}
impl Iterator for WalkDir {
	type Item = std::path::PathBuf;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(mut iter) = self.iter.take() else { return None; };
			let Some(item) = iter.next() else {
				// current entry has finished
				self.iter = self.stack.pop();
				continue;
			};
			let Ok(entry) = item else {
				self.iter = Some(iter);
				continue;
			};
			let Ok(metadata) = entry.metadata() else {
				self.iter = Some(iter);
				continue;
			};
			if metadata.is_dir() {
				let Ok(entry_iter) = std::fs::read_dir(entry.path()) else {
					self.iter = Some(iter);
					continue;
				};
				self.stack.push(iter);
				self.iter = Some(entry_iter);
				continue;
			}
			if !metadata.is_file() {
				self.iter = Some(iter);
				continue;
			}
			self.iter = Some(iter);
			return Some(entry.path());
		}
	}
}
