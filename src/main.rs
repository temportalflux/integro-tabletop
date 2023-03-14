use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
};
use yew::prelude::*;
use yew_hooks::use_is_first_mount;

use crate::system::dnd5e::{data::bounded::BoundValue, DnD5e};

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

fn create_character() -> system::dnd5e::data::character::Persistent {
	use enum_map::enum_map;
	use path_map::PathMap;
	use system::dnd5e::{
		content::*,
		data::{
			character::{NamedGroups, Persistent},
			mutator, Description, *,
		},
		Value,
	};
	use utility::Selector;
	Persistent {
		description: Description {
			name: "Fauxpaul".into(),
			..Default::default()
		},
		ability_scores: enum_map! {
			Ability::Strength => Score(12),
			Ability::Dexterity => Score(15),
			Ability::Constitution => Score(13),
			Ability::Intelligence => Score(17),
			Ability::Wisdom => Score(9),
			Ability::Charisma => Score(11),
		},
		named_groups: NamedGroups {
			lineage: vec![
				lineage::changeling::shapechanger(),
				lineage::changeling::voice_changer(),
			],
			upbringing: vec![upbringing::incognito()],
			background: vec![background::anthropologist()],
		},
		classes: vec![class::barbarian::barbarian(10, None)],
		feats: vec![Feature {
			name: "Custom Feat".into(),
			mutators: vec![
				mutator::AddProficiency::SavingThrow(Ability::Charisma).into(),
				mutator::AddSavingThrowModifier {
					ability: Some(Ability::Charisma),
					target: Some("Magic".into()),
				}
				.into(),
				//mutator::Speed("Flying".into(), 10).into(),
				//mutator::Sense { name: "Darkvision".into(), operation: BoundValue::Minimum(30) }.into(),
				mutator::Sense {
					name: "Tremorsense".into(),
					argument: BoundValue::Minimum(60),
				}
				.into(),
				mutator::AddDefense {
					defense: mutator::Defense::Resistance,
					damage_type: Some(Value::Fixed(DamageType::Cold)),
					..Default::default()
				}
				.into(),
				mutator::AddDefense {
					defense: mutator::Defense::Immunity,
					damage_type: Some(Value::Fixed(DamageType::Acid)),
					..Default::default()
				}
				.into(),
				mutator::AddDefense {
					defense: mutator::Defense::Vulnerability,
					damage_type: Some(Value::Fixed(DamageType::Fire)),
					..Default::default()
				}
				.into(),
				mutator::AddProficiency::Skill(
					Selector::Specific(Skill::Stealth),
					proficiency::Level::Double,
				)
				.into(),
			],
			..Default::default()
		}
		.into()],
		selected_values: PathMap::from([
			(
				PathBuf::from("Incognito/AbilityScoreIncrease"),
				"CON".into(),
			),
			(
				PathBuf::from("Incognito/GoodWithPeople"),
				"Deception".into(),
			),
			(
				PathBuf::from("Incognito/Languages/langA"),
				"Draconic".into(),
			),
			(
				PathBuf::from("Incognito/Languages/langB"),
				"Undercommon".into(),
			),
			(
				PathBuf::from("Anthropologist/Languages/langA"),
				"Sylvan".into(),
			),
			(
				PathBuf::from("Anthropologist/Languages/langB"),
				"Elvish".into(),
			),
			(
				PathBuf::from("Barbarian/level01/skillA"),
				"Intimidation".into(),
			),
			(
				PathBuf::from("Barbarian/level01/skillB"),
				"Athletics".into(),
			),
			(PathBuf::from("Barbarian/level01/hit_points"), "12".into()),
			(PathBuf::from("Barbarian/level02/hit_points"), "7".into()),
			(PathBuf::from("Barbarian/level03/hit_points"), "3".into()),
			(PathBuf::from("Barbarian/level04/hit_points"), "11".into()),
			(PathBuf::from("Barbarian/level05/hit_points"), "8".into()),
		]),
		inventory: {
			let mut inv = item::Inventory::new();
			inv.insert(items::weapon::dagger());
			inv.insert(items::travelers_clothes());
			inv.insert(items::goggles_of_night());
			inv.insert(item::Item {
				name: "Wings of the Owl".into(),
				kind: item::ItemKind::Equipment(item::equipment::Equipment {
					modifiers: vec![
						mutator::Speed {
							name: "Flying".into(),
							argument: BoundValue::Minimum(40),
						}
						.into(),
						mutator::AddProficiency::Skill(
							Selector::Specific(Skill::Perception),
							proficiency::Level::Half,
						)
						.into(),
						mutator::AddSkillModifier {
							skill: Skill::Perception,
							modifier: roll::Modifier::Advantage,
							criteria: Some("when using sight".into()),
						}
						.into(),
					],
					..Default::default()
				}),
				..Default::default()
			});
			inv.insert(items::armor::leather());
			inv.insert(items::armor::splint());
			inv.insert(items::armor::shield());
			inv
		},
		conditions: Vec::new(),
		hit_points: Default::default(),
		inspiration: false,
	}
}

#[function_component]
fn App() -> Html {
	use system::dnd5e;
	let character = create_character();
	let show_browser = use_state_eq(|| false);
	let comp_reg = use_memo(|_| dnd5e::component_registry(), ());
	let node_reg = use_memo(|_| dnd5e::node_registry(), ());
	let modules = use_memo(|_| vec!["basic-rules", "elf-and-orc"], ());
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
						let Ok(insert_callback) = comp_factory.add_from_kdl(node, source_id.clone(), &node_reg) else { continue; };
						(insert_callback)(&mut system);
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
		false => html! {<system::dnd5e::components::CharacterSheetPage {character} />},
		true => html! {},
	};

	return html! {<>
		<header>
			<nav class="navbar navbar-expand-lg sticky-top bg-body-tertiary">
				<div class="container-fluid">
					<a class="navbar-brand" href="/">{"Tabletop Tools"}</a>
					<button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navContent" aria-controls="navContent" aria-expanded="false" aria-label="Toggle navigation">
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
		log::debug!("Parsing {:?}", source_id.to_string());
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
			match comp_factory.add_from_kdl(node, source_id.clone(), &node_reg) {
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
			let Ok(resp) = reqwest::get(uri).await else {
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
