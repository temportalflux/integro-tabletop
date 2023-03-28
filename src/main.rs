use anyhow::Context;
use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	str::FromStr,
};
use yew::prelude::*;
use yew_hooks::use_is_first_mount;

pub mod bootstrap;
pub mod components;
pub mod data;
pub mod kdl_ext;
pub mod logging;
pub mod path_map;
pub mod system;
pub mod theme;
pub mod utility;

#[derive(Clone, PartialEq)]
pub struct Compiled<T> {
	wrapped: T,
	update_root_channel: UseStateHandle<()>,
}
impl<T> std::ops::Deref for Compiled<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.wrapped
	}
}
impl<T> Compiled<T> {
	pub fn update_root(&self) {
		self.update_root_channel.set(());
	}
}

fn create_character(system: &system::dnd5e::DnD5e) -> system::dnd5e::data::character::Persistent {
	use system::{
		core::SourceId,
		dnd5e::data::{character::Persistent, Ability, CurrencyKind, Description, Wallet},
	};
	let mut persistent = Persistent {
		description: Description {
			name: "Sid the Squid".into(),
			..Default::default()
		},
		..Default::default()
	};
	persistent.classes.push({
		let mut class = system
			.classes
			.get(&SourceId::from_str("local://basic-rules@dnd5e/class/barbarian.kdl").unwrap())
			.unwrap()
			.clone();
		class.levels.truncate(3); // level 3
		class
	});
	persistent.named_groups.background.push(
		system
			.backgrounds
			.get(&SourceId::from_str("local://basic-rules@dnd5e/background/folkHero.kdl").unwrap())
			.unwrap()
			.clone(),
	);
	persistent.named_groups.lineage.push(
		system
			.lineages
			.get(
				&SourceId::from_str("local://elf-and-orc@dnd5e/lineage/dwarven/dwarven.kdl")
					.unwrap(),
			)
			.unwrap()
			.clone(),
	);
	persistent.named_groups.lineage.push(
		system
			.lineages
			.get(&SourceId::from_str("local://elf-and-orc@dnd5e/lineage/dwarven/gray.kdl").unwrap())
			.unwrap()
			.clone(),
	);
	persistent.named_groups.upbringing.push(
		system
			.upbringings
			.get(&SourceId::from_str("local://elf-and-orc@dnd5e/upbringing/abjurer.kdl").unwrap())
			.unwrap()
			.clone(),
	);
	persistent.ability_scores[Ability::Strength] = 15;
	persistent.ability_scores[Ability::Dexterity] = 10;
	persistent.ability_scores[Ability::Constitution] = 14;
	persistent.ability_scores[Ability::Intelligence] = 12;
	persistent.ability_scores[Ability::Wisdom] = 8;
	persistent.ability_scores[Ability::Charisma] = 13;
	*persistent.inventory.wallet_mut() = Wallet::from([
		(3, CurrencyKind::Platinum),
		(16, CurrencyKind::Gold),
		(4, CurrencyKind::Electrum),
		(30, CurrencyKind::Silver),
		(152, CurrencyKind::Copper),
	]);
	persistent
}

#[function_component]
fn App() -> Html {
	use system::dnd5e::{self, DnD5e};
	let show_browser = use_state_eq(|| false);
	let comp_reg = use_memo(|_| dnd5e::component_registry(), ());
	let node_reg = use_memo(|_| dnd5e::node_registry(), ());
	let modules = use_memo(|_| vec!["basic-rules", "elf-and-orc", "wotc-toa"], ());
	let system = use_state(|| DnD5e::default());

	let content_loader = yew_hooks::use_async({
		let comp_reg = comp_reg.clone();
		let node_reg = node_reg.clone();
		let modules = modules.clone();
		let system_state = system.clone();
		async move {
			let mut system = DnD5e::default();
			for module in &*modules {
				log::info!("Loading content module {module:?}");
				let sources = fetch_local_module(*module, &["dnd5e"]).await?;
				for (mut source_id, content) in sources {
					let Ok(document) = content.parse::<kdl::KdlDocument>() else { continue; };
					for (idx, node) in document.nodes().iter().enumerate() {
						source_id.node_idx = idx;
						let Some(comp_factory) = comp_reg.get_factory(node.name().value()).cloned() else { continue; };
						match comp_factory
							.add_from_kdl(node, source_id.clone(), &node_reg)
							.with_context(|| format!("Failed to parse {:?}", source_id.to_string()))
						{
							Ok(insert_callback) => {
								(insert_callback)(&mut system);
							}
							Err(err) => {
								log::error!(target: *module, "{err:?}");
							}
						}
					}
				}
			}
			system_state.set(system);
			Ok(()) as Result<(), std::sync::Arc<anyhow::Error>>
		}
	});
	if use_is_first_mount() {
		content_loader.run();
	}

	let initial_character = use_state(|| None);
	use_effect_with_deps(
		{
			let initial_character = initial_character.clone();
			move |(system, has_loaded): &(UseStateHandle<DnD5e>, bool)| {
				if *has_loaded {
					initial_character.set(Some(create_character(&**system)));
				}
			}
		},
		(system.clone(), content_loader.data.is_some()),
	);

	let open_character = Callback::from({
		let show_browser = show_browser.clone();
		move |_| {
			show_browser.set(false);
		}
	});
	let open_content = Callback::from({
		let show_browser = show_browser.clone();
		move |_| {
			show_browser.set(true);
		}
	});

	let content = match *show_browser {
		false => match &*initial_character {
			None => html! {},
			Some(character) => {
				let character = character.clone();
				html! {<system::dnd5e::components::CharacterSheetPage {character} />}
			}
		},
		true => html! {},
	};

	return html! {<>
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<a class="navbar-brand" href="/">{"Tabletop Tools"}</a>
					<button
						class="navbar-toggler" type="button"
						data-bs-toggle="collapse" data-bs-target="#navContent"
						aria-controls="navContent" aria-expanded="false" aria-label="Toggle navigation"
					>
						<span class="navbar-toggler-icon"></span>
					</button>
					<div class="collapse navbar-collapse" id="navContent">
						<ul class="navbar-nav">
							<li class="nav-item">
								<a class="nav-link" onclick={open_character}>{"My Characters"}</a>
							</li>
							<li class="nav-item">
								<a class="nav-link" onclick={open_content}>{"Content Browser"}</a>
							</li>
						</ul>
						<ul class="navbar-nav flex-row flex-wrap ms-md-auto">
							<theme::Dropdown />
						</ul>
					</div>
				</div>
			</nav>
		</header>
		<ContextProvider<UseStateHandle<DnD5e>> context={system.clone()}>
			{content}
		</ContextProvider<UseStateHandle<DnD5e>>>
	</>};
}

#[cfg(target_family = "wasm")]
fn main() {
	logging::wasm::init(logging::wasm::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[cfg(target_family = "windows")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	use crate::system::{
		core::{ModuleId, SourceId},
		dnd5e,
	};
	use anyhow::Context;
	use std::collections::BTreeMap;

	let _ = logging::console::init("tabletop-tools", &[]);

	let comp_reg = dnd5e::component_registry();
	let node_reg = dnd5e::node_registry();
	let mut system = dnd5e::DnD5e::default();

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
					module: ModuleId::Local {
						name: module_id.clone(),
					},
					system: system_id.clone(),
					path: item_relative_path.to_owned(),
					version: None,
					node_idx: 0,
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
			.context("Invalid KDL format")?;
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let node_name = node.name().value();
			let Some(comp_factory) = comp_reg.get_factory(node_name).cloned() else {
				log::error!("Failed to find factory to deserialize node \"{node_name}\".");
				continue;
			};
			let insert_parsed = comp_factory.add_from_kdl(node, source_id.clone(), &node_reg);
			let insert_parsed =
				insert_parsed.with_context(|| format!("while parsing {}", source_id.to_string()));
			match insert_parsed {
				Ok(insert_callback) => (insert_callback)(&mut system),
				Err(err) => {
					log::error!("Failed to deserialize entry: {err:?}");
				}
			}
		}
	}

	Ok(())
}

async fn fetch_local_module(
	name: &'static str,
	systems: &[&'static str],
) -> anyhow::Result<BTreeMap<system::core::SourceId, String>> {
	use crate::system::core::{ModuleId, SourceId};
	// This is a temporary flow until github oauth is up and running.
	// Expects:
	// - a provided module with the folder-name `name` is at `./modules/{name}`.
	// - a module has system directories as root folders.
	//   e.g. all files for `dnd5e` live at `./modules/{name}/dnd5e/..`
	// - each system has an index file at `./modules/{name}/{system}/index`,
	//   where each line in the file is a relative path from the system folder to the resource.
	//   (This will not be needed when github fetching is up and running.)
	static ROOT_URL: &'static str = "http://localhost:8080/modules";
	let mut content = BTreeMap::new();
	for system in systems {
		let index_uri = format!("{ROOT_URL}/{name}/{system}/index");
		let Ok(resp) = reqwest::get(index_uri).await
		else {
			return Err(MissingResource(name, system, "index".into()).into());
		};
		let index_content = resp.text().await?;
		for line in index_content.lines() {
			let uri = format!("{ROOT_URL}/{name}/{system}/{line}");
			let Ok(resp) = reqwest::get(uri.clone()).await else {
				return Err(MissingResource(name, system, line.into()).into());
			};
			let source_id = SourceId {
				module: ModuleId::Local {
					name: name.to_owned(),
				},
				system: (*system).to_owned(),
				path: PathBuf::from(line),
				version: None,
				node_idx: 0,
			};
			content.insert(source_id, resp.text().await?);
		}
	}
	Ok(content)
}

#[derive(thiserror::Error, Debug)]
#[error("Missing resource in module {0} for system {1} at path {2}.")]
struct MissingResource(&'static str, &'static str, String);

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
	fn new(path: impl AsRef<Path>) -> Self {
		Self {
			iter: std::fs::read_dir(path).ok(),
			stack: Vec::new(),
		}
	}
}
impl Iterator for WalkDir {
	type Item = PathBuf;

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
