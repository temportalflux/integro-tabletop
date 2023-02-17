use yew::prelude::*;

pub mod bootstrap;
pub mod components;
pub mod data;
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
		hardcoded::*,
		modifier::{self, AddSavingThrow},
		*,
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
		lineages: [Some(changeling1()), Some(changeling2())],
		upbringing: Some(incognito()),
		background: Some(anthropologist()),
		classes: Vec::new(),
		feats: vec![Feature {
			name: "Custom Feat".into(),
			modifiers: vec![
				AddSavingThrow::Proficiency(Ability::Charisma).into(),
				AddSavingThrow::Advantage(Ability::Charisma, "Magic".into()).into(),
				//modifier::AddMaxSpeed("Flying".into(), 10).into(),
				modifier::AddMaxSense("Darkvision".into(), 30).into(),
				//modifier::AddMaxSense("Tremorsense".into(), 60).into(),
				modifier::AddDefense(modifier::Defense::Resistant, "Cold".into()).into(),
				modifier::AddDefense(modifier::Defense::Immune, "Acid".into()).into(),
				modifier::AddDefense(modifier::Defense::Vulnerable, "Fire".into()).into(),
			],
			..Default::default()
		}],
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
		]),
		inventory: {
			use character::inventory::*;
			use roll::*;
			let mut inv = Inventory::new();
			inv.insert(Item {
				name: "Dagger".into(),
				description: None,
				weight: 1,
				worth: 200, // in copper
				kind: ItemKind::Equipment(Equipment {
					weapon: Some(Weapon {
						kind: WeaponType::Simple,
						damage: Roll {
							amount: 1,
							die: Die::D4,
						},
						damage_type: "piercing".into(),
						properties: vec![
							Property::Light,
							Property::Finesse,
							Property::Thrown(20, 60),
						],
						range: None,
					}),
					..Default::default()
				}),
				..Default::default()
			});
			inv.insert(Item {
				name: "Traveler's Clothes".into(),
				description: Some(
					"This set of clothes could consist of boots, a wool skirt or breeches, \
				a sturdy belt, a shirt (perhaps with a vest or jacket), and an ample cloak with a hood."
						.into(),
				),
				weight: 4,
				worth: 200,
				..Default::default()
			});
			inv.insert(Item {
				name: "Goggles of Night".into(),
				description: Some(
					"While wearing these dark lenses, you have darkvision \
				out to a range of 60 feet. If you already have darkvision, wearing the \
				goggles increases its range by 60 feet."
						.into(),
				),
				kind: ItemKind::Equipment(Equipment {
					modifiers: vec![modifier::AddMaxSense("Darkvision".into(), 60).into()],
					..Default::default()
				}),
				..Default::default()
			});
			inv.insert(Item {
				name: "Wings of the Owl".into(),
				kind: ItemKind::Equipment(Equipment {
					modifiers: vec![
						modifier::AddMaxSpeed("Flying".into(), 40).into(),
						modifier::AddSkill {
							skill: modifier::Selector::Specific(Skill::Perception),
							proficiency: proficiency::Level::Half,
						}
						.into(),
						modifier::AddSkillModifier {
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
			inv.insert(Item {
				name: "Splint".into(),
				description: None,
				weight: 60,
				worth: 20000, // in copper
				notes: "".into(),
				kind: ItemKind::Equipment(Equipment {
					modifiers: vec![modifier::AddSkillModifier {
						skill: Skill::Stealth,
						modifier: roll::Modifier::Disadvantage,
						criteria: None,
					}
					.into()],
					armor: Some(Armor {
						kind: ArmorType::Heavy,
						base_score: 17,
						ability_modifier: None,
						max_ability_bonus: None,
						min_strength_score: Some(15),
					}),
					..Default::default()
				}),
			});
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
										{"Features & Traits"}
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
