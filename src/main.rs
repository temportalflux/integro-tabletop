use yew::prelude::*;

pub mod bootstrap;
pub mod components;
pub mod data;
pub mod path_map;
pub mod system;
pub mod theme;

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

fn create_character() -> system::dnd5e::character::Character {
	use enum_map::enum_map;
	use std::{collections::HashMap, path::PathBuf};
	use system::dnd5e::{
		character::{Character, Description},
		content::*,
		mutator, *,
	};
	Character {
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
				mutator::AddMaxSense("Darkvision".into(), 30).into(),
				//mutator::AddMaxSense("Tremorsense".into(), 60).into(),
				mutator::AddDefense(mutator::Defense::Resistant, "Cold".into()).into(),
				mutator::AddDefense(mutator::Defense::Immune, "Acid".into()).into(),
				mutator::AddDefense(mutator::Defense::Vulnerable, "Fire".into()).into(),
			],
			..Default::default()
		}
		.into()],
		selected_values: HashMap::from([
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
			(PathBuf::from("Barbarian/skillA"), "Intimidation".into()),
			(PathBuf::from("Barbarian/skillB"), "Athletics".into()),
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
							skill: mutator::Selector::Specific(Skill::Perception),
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
			inv
		},
		conditions: Vec::new(),
		hit_points: (0, 0),
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
		<CharacterSheetPage {character} />
	</>};
}

#[derive(Clone, PartialEq, Properties)]
struct CharacterSheetPageProps {
	character: system::dnd5e::character::Character,
}

#[function_component]
fn CharacterSheetPage(CharacterSheetPageProps { character }: &CharacterSheetPageProps) -> Html {
	use components::*;
	use data::ContextMut;
	use system::dnd5e::character::State;
	use system::dnd5e::Ability;

	let character = ContextMut::<State>::from(use_reducer({
		let character = character.clone();
		move || State::from(character)
	}));

	html! {
		<ContextProvider<ContextMut<State>> context={character.clone()}>
			<div class="container overflow-hidden" style="--theme-frame-color: #BA90CB; --theme-frame-color-muted: #BA90CB80; --theme-roll-modifier: #ffffff;">
				<div class="row" style="--bs-gutter-x: 10px;">
					<div class="col-md-auto">

						<div class="row m-0" style="--bs-gutter-x: 0;">
							<div class="col">
								<ability::Score ability={Ability::Strength} />
								<ability::Score ability={Ability::Dexterity} />
								<ability::Score ability={Ability::Constitution} />
							</div>
							<div class="col">
								<ability::Score ability={Ability::Intelligence} />
								<ability::Score ability={Ability::Wisdom} />
								<ability::Score ability={Ability::Charisma} />
							</div>
						</div>

						<ability::SavingThrowContainer />
						<Proficiencies />

					</div>
					<div class="col-md-auto">

						<div class="row m-0 justify-content-center">
							<div class="col p-0">
								<AnnotatedNumberCard header={"Proficiency"} footer={"Bonus"}>
									<AnnotatedNumber value={character.proficiency_bonus()} show_sign={true} />
								</AnnotatedNumberCard>
							</div>
							<div class="col p-0">
								<AnnotatedNumberCard header={"Initiative"} footer={"Bonus"}>
									<AnnotatedNumber value={character.initiative_bonus()} show_sign={true} />
								</AnnotatedNumberCard>
							</div>
							<div class="col p-0">
								<AnnotatedNumberCard header={"Armor"} footer={"Class"}>
									<AnnotatedNumber value={character.armor_class()} />
								</AnnotatedNumberCard>
							</div>
						</div>

						<div id="skills-container" class="card" style="min-width: 320px; border-color: var(--theme-frame-color);">
							<div class="card-body" style="padding: 5px;">
								<ability::SkillTable />
							</div>
						</div>

					</div>
					<div class="col">
						<div class="row m-0" style="--bs-gutter-x: 0;">
							<div class="col">
								{"TODO: Inspiration"}
								<SpeedAndSenses />
							</div>
							<div class="col-auto">
								<HitPoints />
							</div>
						</div>

						<div class="card m-2" style="height: 550px;">
							<div class="card-body" style="padding: 5px;">
								<Nav root_classes={"onesheet-tabs"} disp={NavDisplay::Tabs} default_tab_id={"inventory"}>
									<TabContent id="actions" title={html! {{"Actions"}}}>
										<panel::Actions />
									</TabContent>
									<TabContent id="spells" title={html! {{"Spells"}}}>
										{"Spells"}
									</TabContent>
									<TabContent id="inventory" title={html! {{"Inventory"}}}>
										<panel::Inventory />
									</TabContent>
									<TabContent id="features" title={html! {{"Features & Traits"}}}>
										<panel::Features />
									</TabContent>
									<TabContent id="description" title={html! {{"Description"}}}>
										{"Description"}
									</TabContent>
								</Nav>
							</div>
						</div>
					</div>
				</div>
			</div>
		</ContextProvider<ContextMut<State>>>
	}
}

fn main() {
	wasm_logger::init(wasm_logger::Config::default());
	yew::Renderer::<App>::new().render();
}
