use std::path::{Path, PathBuf};
use yew::prelude::*;

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
		data::{character::Persistent, mutator, Description, *},
		Value,
	};
	use utility::Selector;
	Persistent {
		description: Description {
			name: "Fauxpaul".into(),
			pronouns: "".into(),
		},
		ability_scores: enum_map! {
			Ability::Strength => Score(12),
			Ability::Dexterity => Score(15),
			Ability::Constitution => Score(13),
			Ability::Intelligence => Score(17),
			Ability::Wisdom => Score(9),
			Ability::Charisma => Score(11),
		},
		lineages: [
			Some(lineage::changeling::shapechanger()),
			Some(lineage::changeling::voice_changer()),
		],
		upbringing: Some(upbringing::incognito()),
		background: Some(background::anthropologist()),
		classes: vec![class::barbarian::barbarian(10, None)],
		feats: vec![Feature {
			name: "Custom Feat".into(),
			mutators: vec![
				mutator::AddSavingThrow(Ability::Charisma).into(),
				mutator::AddSavingThrowModifier {
					ability: Some(Ability::Charisma),
					target: Some("Magic".into()),
				}
				.into(),
				//mutator::AddMaxSpeed("Flying".into(), 10).into(),
				//mutator::AddMaxSense("Darkvision".into(), 30).into(),
				mutator::AddMaxSense("Tremorsense".into(), 60).into(),
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
				mutator::AddSkill {
					skill: Selector::Specific(Skill::Stealth),
					proficiency: proficiency::Level::Double,
				}
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
						mutator::AddMaxSpeed("Flying".into(), 40).into(),
						mutator::AddSkill {
							skill: Selector::Specific(Skill::Perception),
							proficiency: proficiency::Level::Half,
						}
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
	let character = create_character();

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
							<li class="nav-item">{"My Characters"}</li>
						</ul>
						<ul class="navbar-nav flex-row flex-wrap ms-md-auto">
							<theme::Dropdown />
						</ul>
					</div>
				</div>
			</nav>
		</header>
		<system::dnd5e::components::CharacterSheetPage {character} />
	</>};
}

#[cfg(target_family = "wasm")]
fn main() {
	logging::wasm::init(logging::wasm::Config::default().prefer_target());
	yew::Renderer::<App>::new().render();
}

#[cfg(target_family = "windows")]
fn main() -> anyhow::Result<()> {
	let _ = logging::console::init("tabletop-tools", &[]);

	let mut system_reg = system::core::SystemRegistry::default();
	system_reg.register(system::dnd5e::DnD5e::new());

	for item in WalkDir::new("./modules") {
		let Some(ext) = item.extension() else { continue; };
		if ext.to_str() != Some("kdl") {
			continue;
		}
		let Ok(content) = std::fs::read_to_string(&item) else { continue; };
		if let Err(err) = insert_system_document(&system_reg, &content) {
			log::error!("Failed to parse module document {item:?}: {err:?}");
		}
	}

	Ok(())
}

#[cfg(target_family = "windows")]
fn insert_system_document(
	system_reg: &system::core::SystemRegistry,
	content: &str,
) -> anyhow::Result<()> {
	use anyhow::Context;
	use kdl_ext::DocumentQueryExt;
	let document = content
		.parse::<kdl::KdlDocument>()
		.context("Invalid KDL format")?;
	let system_id = document.query_str("system", 0)?;
	let mut system = system_reg
		.get(system_id)
		.ok_or(GeneralError(format!("System {system_id:?} not found")))?;
	system.insert_document(document);
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
