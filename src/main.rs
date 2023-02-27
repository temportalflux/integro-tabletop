use yew::prelude::*;

pub mod bootstrap;
pub mod components;
pub mod data;
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
	use std::path::PathBuf;
	use system::dnd5e::{
		content::*,
		data::{character::Persistent, mutator, Description, *},
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
				mutator::AddSavingThrow::Proficiency(Ability::Charisma).into(),
				mutator::AddSavingThrow::Advantage(Ability::Charisma, Some("Magic".into())).into(),
				//mutator::AddMaxSpeed("Flying".into(), 10).into(),
				//mutator::AddMaxSense("Darkvision".into(), 30).into(),
				mutator::AddMaxSense("Tremorsense".into(), 60).into(),
				mutator::AddDefense(mutator::Defense::Resistant, "Cold".into()).into(),
				mutator::AddDefense(mutator::Defense::Immune, "Acid".into()).into(),
				mutator::AddDefense(mutator::Defense::Vulnerable, "Fire".into()).into(),
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
		hit_points: (41, 0),
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

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<App>::new().render();
}
