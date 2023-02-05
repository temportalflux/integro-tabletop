use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	str::FromStr,
};

use enum_map::{Enum, EnumMap};
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};

#[derive(EnumSetType, PartialOrd, Ord, Hash, Enum, Debug)]
pub enum Ability {
	Strength,
	Dexterity,
	Constitution,
	Intelligence,
	Wisdom,
	Charisma,
}

impl Ability {
	pub fn long_name(&self) -> &'static str {
		match self {
			Self::Strength => "Strength",
			Self::Dexterity => "Dexterity",
			Self::Constitution => "Constitution",
			Self::Intelligence => "Intelligence",
			Self::Wisdom => "Wisdom",
			Self::Charisma => "Charisma",
		}
	}

	pub fn abbreviated_name(&self) -> &'static str {
		match self {
			Self::Strength => "str",
			Self::Dexterity => "dex",
			Self::Constitution => "con",
			Self::Intelligence => "int",
			Self::Wisdom => "wis",
			Self::Charisma => "cha",
		}
	}
}

impl FromStr for Ability {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"str" | "strength" => Ok(Self::Strength),
			"dex" | "dexterity" => Ok(Self::Dexterity),
			"con" | "constitution" => Ok(Self::Constitution),
			"int" | "intelligence" => Ok(Self::Intelligence),
			"wis" | "wisdom" => Ok(Self::Wisdom),
			"cha" | "charisma" => Ok(Self::Charisma),
			_ => Err(()),
		}
	}
}

#[derive(Clone)]
pub struct Character {
	description: Description,
	ability_scores: EnumMap<Ability, i32>,
	culture: Culture,
	selected_values: HashMap<PathBuf, String>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct CompiledStats {
	missing_selections: Vec<PathBuf>,
	ability_scores: EnumMap<Ability, i32>,
	skills: EnumMap<Skill, ProficiencyLevel>,
	languages: Vec<String>,
	life_expectancy: i32,
	max_height: (i32, RollSet),
}

impl Character {
	pub fn compile_stats(&self) -> CompiledStats {
		let mut stats = CompiledStats::default();
		stats.ability_scores = self.ability_scores;

		self.culture
			.apply_modifiers(&self, &mut stats, PathBuf::from("culture"));

		stats
	}

	pub fn get_selection(&self, stats: &mut CompiledStats, scope: &Path) -> Option<&str> {
		let selection = self.selected_values.get(scope).map(String::as_str);
		if selection.is_none() {
			stats.missing_selections.push(scope.to_owned());
		}
		selection
	}
}

#[derive(Clone, PartialEq)]
pub struct Description {
	name: String,
	pronouns: String,
}

#[derive(Clone, PartialEq)]
pub struct Culture {
	lineages: [Lineage; 2],
	upbringing: Upbringing,
}
impl ModifierTraitContainer for Culture {
	fn apply_modifiers(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		for lineage in &self.lineages {
			lineage.apply_modifiers(char, stats, scope.join(&lineage.id()));
		}
		self.upbringing
			.apply_modifiers(char, stats, scope.join(&self.upbringing.id()));
	}
}

#[derive(Default, Clone, PartialEq)]
pub struct Lineage {
	name: String,
	description: String,
	features: Vec<Feature>,
}
impl Lineage {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}
}
impl ModifierTraitContainer for Lineage {
	fn apply_modifiers(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		for feat in &self.features {
			feat.apply_modifiers(char, stats, scope.join(&feat.id()));
		}
	}
}

#[derive(Default, Clone, PartialEq)]
pub struct Upbringing {
	name: String,
	description: String,
	features: Vec<Feature>,
}
impl Upbringing {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}
}
impl ModifierTraitContainer for Upbringing {
	fn apply_modifiers(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		for feat in &self.features {
			feat.apply_modifiers(char, stats, scope.join(&feat.id()));
		}
	}
}

#[derive(Default, Clone)]
pub struct Feature {
	name: String,
	description: String,
	action: Option<Action>,
	modifiers: Vec<Box<dyn ModifierTrait + 'static>>,
}
impl Feature {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}
}
impl PartialEq for Feature {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.description == other.description
			&& self.action == other.action
	}
}
impl ModifierTraitContainer for Feature {
	fn apply_modifiers(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		for modifier in &self.modifiers {
			modifier.apply(
				char,
				stats,
				match modifier.scope_id() {
					Some(id) => scope.join(id),
					None => scope.clone(),
				},
			);
		}
	}
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Action {
	Full,
	Bonus,
	Reaction,
}

#[derive(EnumSetType, PartialOrd, Enum, Serialize, Deserialize, Debug)]
pub enum Skill {
	Acrobatics,
	AnimalHandling,
	Arcana,
	Athletics,
	Deception,
	History,
	Insight,
	Intimidation,
	Investigation,
	Medicine,
	Nature,
	Perception,
	Performance,
	Persuasion,
	Religion,
	SleightOfHand,
	Stealth,
	Survival,
}

impl Skill {
	pub fn ability(&self) -> Ability {
		match self {
			Self::Acrobatics => Ability::Dexterity,
			Self::AnimalHandling => Ability::Wisdom,
			Self::Arcana => Ability::Intelligence,
			Self::Athletics => Ability::Strength,
			Self::Deception => Ability::Charisma,
			Self::History => Ability::Intelligence,
			Self::Insight => Ability::Wisdom,
			Self::Intimidation => Ability::Charisma,
			Self::Investigation => Ability::Intelligence,
			Self::Medicine => Ability::Wisdom,
			Self::Nature => Ability::Intelligence,
			Self::Perception => Ability::Wisdom,
			Self::Performance => Ability::Charisma,
			Self::Persuasion => Ability::Charisma,
			Self::Religion => Ability::Intelligence,
			Self::SleightOfHand => Ability::Dexterity,
			Self::Stealth => Ability::Dexterity,
			Self::Survival => Ability::Wisdom,
		}
	}

	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Acrobatics => "Acrobatics",
			Self::AnimalHandling => "Animal Handling",
			Self::Arcana => "Arcana",
			Self::Athletics => "Athletics",
			Self::Deception => "Deception",
			Self::History => "History",
			Self::Insight => "Insight",
			Self::Intimidation => "Intimidation",
			Self::Investigation => "Investigation",
			Self::Medicine => "Medicine",
			Self::Nature => "Nature",
			Self::Perception => "Perception",
			Self::Performance => "Performance",
			Self::Persuasion => "Persuasion",
			Self::Religion => "Religion",
			Self::SleightOfHand => "Sleight of Hand",
			Self::Stealth => "Stealth",
			Self::Survival => "Survival",
		}
	}
}

impl FromStr for Skill {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"acrobatics" => Ok(Self::Acrobatics),
			"animalhandling" => Ok(Self::AnimalHandling),
			"arcana" => Ok(Self::Arcana),
			"athletics" => Ok(Self::Athletics),
			"deception" => Ok(Self::Deception),
			"history" => Ok(Self::History),
			"insight" => Ok(Self::Insight),
			"intimidation" => Ok(Self::Intimidation),
			"investigation" => Ok(Self::Investigation),
			"medicine" => Ok(Self::Medicine),
			"nature" => Ok(Self::Nature),
			"perception" => Ok(Self::Perception),
			"performance" => Ok(Self::Performance),
			"persuasion" => Ok(Self::Persuasion),
			"religion" => Ok(Self::Religion),
			"sleightofhand" => Ok(Self::SleightOfHand),
			"stealth" => Ok(Self::Stealth),
			"survival" => Ok(Self::Survival),
			_ => Err(()),
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum ModifierTarget {
	Placeholder(String),
	Choice(usize, Vec<String>),
}

#[derive(Clone, Copy, PartialEq)]
pub enum ModifierEffect {
	Bonus(ModifierBonus),
	Proficiency(ProficiencyLevel),
}

#[derive(Clone, Copy, PartialEq)]
pub enum ModifierBonus {
	Value(i32),
	Roll(Roll),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Roll {
	amount: i32,
	die: Die,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct RollSet(EnumMap<Die, i32>);
impl RollSet {
	pub fn push(&mut self, roll: Roll) {
		self.0[roll.die] += roll.amount;
	}
}

#[derive(Debug, Enum, EnumSetType)]
pub enum Die {
	D4,
	D6,
	D8,
	D10,
	D12,
	D20,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ProficiencyLevel {
	None,
	Half,
	Full,
	Double,
}
impl Default for ProficiencyLevel {
	fn default() -> Self {
		Self::None
	}
}
impl ProficiencyLevel {
	pub fn as_display_name(&self) -> &'static str {
		match self {
			Self::None => "Not Proficient",
			Self::Half => "Half Proficient",
			Self::Full => "Proficient",
			Self::Double => "Expertise",
		}
	}

	pub fn bonus_multiplier(&self) -> f32 {
		match self {
			Self::None => 0.0,
			Self::Half => 0.5,
			Self::Full => 1.0,
			Self::Double => 2.0,
		}
	}
}
impl Into<yew::prelude::Html> for ProficiencyLevel {
	fn into(self) -> yew::prelude::Html {
		use yew::prelude::*;
		match self {
			Self::None => html! { <i class="fa-regular fa-circle" /> },
			Self::Half => {
				html! { <i class="fa-solid fa-circle-half-stroke" style="color: var(--theme-frame-color);" /> }
			}
			Self::Full => {
				html! { <i class="fa-solid fa-circle" style="color: var(--theme-frame-color);" /> }
			}
			Self::Double => {
				html! { <i class="fa-regular fa-circle-dot" style="color: var(--theme-frame-color);" /> }
			}
		}
	}
}

impl std::ops::Mul<i32> for ProficiencyLevel {
	type Output = i32;

	fn mul(self, prof_bonus: i32) -> Self::Output {
		let modified = (prof_bonus as f32) * self.bonus_multiplier();
		modified.floor() as i32
	}
}

pub fn changeling_culture() -> Culture {
	let life_expectancy = Feature {
		name: "Age".into(),
		description: "Your life expectancy increases by 50 years.".into(),
		modifiers: vec![Box::new(AddLifeExpectancy(50))],
		..Default::default()
	};
	let size = Feature {
		name: "Size".into(),
		description: "Your height increases by 30 + 1d4 inches.".into(),
		modifiers: vec![
			Box::new(AddMaxHeight::Value(30)),
			Box::new(AddMaxHeight::Roll(Roll {
				amount: 1,
				die: Die::D4,
			})),
		],
		..Default::default()
	};
	Culture {
    lineages: [
			Lineage {
				name: "Changeling I".to_owned(),
				description: "One of your birth parents is a changeling. You can change your appearance at will.".to_owned(),
				features: vec![
					life_expectancy.clone(),
					size.clone(),
					Feature {
						name: "Shapechanger".to_owned(),
						description: "As an action, you can change your appearance. You determine the specifics of the changes, \
						including your coloration, hair length, and sex. You can also adjust your height and weight, \
						but not so much that your size changes. While shapechanged, none of your game statistics change. \
						You can't duplicate the appearance of a creature you've never seen, and you must adopt a form that has \
						the same basic arrangement of limbs that you have. Your voice, clothing, and equipment aren't changed by this trait. \
						You stay in the new form until you use an action to revert to your true form or until you die.".into(),
						action: Some(Action::Full),
						..Default::default()
					},
				],
			},
			Lineage {
				name: "Changeling II".to_owned(),
				description: "One of your birth parents is a changeling. You can perfectly mimic another person's voice.".to_owned(),
				features: vec![
					life_expectancy.clone(),
					size.clone(),
					Feature {
						name: "Voice Change".to_owned(),
						description: "As an action, you can change your voice. You can't duplicate the voice of a creature you've never heard. \
						Your appearance remains the same. You keep your mimicked voice until you use an action to revert to your true voice.".into(),
						action: Some(Action::Full),
						..Default::default()
					},
				],
			},
		],
    upbringing: Upbringing {
			name: "Incognito".into(),
			description: "You were brought up by those who were not what they seemed.".into(),
			features: vec![
				Feature {
					name: "Ability Score Increase".into(),
					description: "Your Charisma score increases by 2. In addition, one ability score of your choice increases by 1.".into(),
					modifiers: vec![
						Box::new(AddAbilityScore {
							ability: Selector::Specific(Ability::Charisma),
							value: 2,
						}),
						Box::new(AddAbilityScore {
							ability: Selector::AnyOf {
								id: None,
								options: EnumSet::only(Ability::Charisma).complement().into_iter().collect(),
							},
							value: 1,
						}),
					],
					..Default::default()
				},
				Feature {
					name: "Good with People".into(),
					description: "You gain proficiency with two of the following skills of your choice: Deception, Insight, Intimidation, and Persuasion.".into(),
					modifiers: vec![
						Box::new(AddSkill {
							skill: Selector::AnyOf {
								id: None,
								options: vec![
									Skill::Deception, Skill::Insight, Skill::Intimidation, Skill::Persuasion,
								],
							},
							proficiency: ProficiencyLevel::Full,
						}),
					],
					..Default::default()
				},
				Feature {
					name: "Languages".into(),
					description: "You can speak, read, and write Common and two other languages of your choice.".into(),
					modifiers: vec![
						Box::new(AddLanguage(Selector::Specific("Common".into()))),
						Box::new(AddLanguage(Selector::Any { id: Some("langA".into()) })),
						Box::new(AddLanguage(Selector::Any { id: Some("langB".into()) })),
					],
					..Default::default()
				},
			],
		},
	}
}

pub trait BoxedCloneModifierTrait {
	fn clone_box<'a>(&self) -> Box<dyn ModifierTrait>;
}
impl<T> BoxedCloneModifierTrait for T
where
	T: ModifierTrait + Clone + 'static,
{
	fn clone_box<'a>(&self) -> Box<dyn ModifierTrait> {
		Box::new(self.clone())
	}
}

pub trait ModifierTrait: BoxedCloneModifierTrait {
	fn scope_id(&self) -> Option<&str> {
		None
	}
	fn apply(&self, _: &Character, _: &mut CompiledStats, _: PathBuf) {}
}
impl Clone for Box<dyn ModifierTrait> {
	fn clone(&self) -> Box<dyn ModifierTrait> {
		self.clone_box()
	}
}

pub trait ModifierTraitContainer {
	fn apply_modifiers(&self, character: &Character, stats: &mut CompiledStats, scope: PathBuf);
}

#[derive(Clone)]
pub enum Selector<T> {
	Specific(T),
	AnyOf { id: Option<String>, options: Vec<T> },
	Any { id: Option<String> },
}
impl<T> Selector<T> {
	pub fn id(&self) -> Option<&str> {
		match self {
			Self::Specific(_) => None,
			Self::AnyOf { id, options: _ } => id.as_ref(),
			Self::Any { id } => id.as_ref(),
		}
		.map(String::as_str)
	}
}

#[derive(Clone)]
pub struct AddAbilityScore {
	ability: Selector<Ability>,
	value: i32,
}
impl ModifierTrait for AddAbilityScore {
	fn scope_id(&self) -> Option<&str> {
		self.ability.id()
	}

	fn apply(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		let ability = match &self.ability {
			Selector::Specific(ability) => Some(*ability),
			_ => match char.get_selection(stats, &scope) {
				Some(value) => Ability::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(ability) = ability {
			stats.ability_scores[ability] += self.value;
		}
	}
}

#[derive(Clone)]
pub struct AddSkill {
	skill: Selector<Skill>,
	proficiency: ProficiencyLevel,
}
impl ModifierTrait for AddSkill {
	fn scope_id(&self) -> Option<&str> {
		self.skill.id()
	}

	fn apply(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		let skill = match &self.skill {
			Selector::Specific(skill) => Some(*skill),
			_ => match char.get_selection(stats, &scope) {
				Some(value) => Skill::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(skill) = skill {
			stats.skills[skill] = stats.skills[skill].max(self.proficiency);
		}
	}
}

#[derive(Clone)]
pub struct AddLanguage(Selector<String>);
impl ModifierTrait for AddLanguage {
	fn scope_id(&self) -> Option<&str> {
		self.0.id()
	}

	fn apply(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		let language = match &self.0 {
			Selector::Specific(language) => Some(language.clone()),
			_ => char.get_selection(stats, &scope).map(str::to_owned),
		};
		if let Some(lang) = language {
			stats.languages.push(lang);
		}
	}
}

#[derive(Clone)]
pub struct AddLifeExpectancy(i32);
impl ModifierTrait for AddLifeExpectancy {
	fn apply(&self, _: &Character, stats: &mut CompiledStats, _: PathBuf) {
		stats.life_expectancy += self.0;
	}
}

#[derive(Clone)]
pub enum AddMaxHeight {
	Value(i32),
	Roll(Roll),
}
impl ModifierTrait for AddMaxHeight {
	fn apply(&self, _: &Character, stats: &mut CompiledStats, _: PathBuf) {
		match self {
			Self::Value(value) => {
				stats.max_height.0 += *value;
			}
			Self::Roll(roll) => {
				stats.max_height.1.push(*roll);
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use enum_map::enum_map;

	#[test]
	fn test_changeling() {
		let character = Character {
			description: Description {
				name: "changeling".into(),
				pronouns: "".into(),
			},
			ability_scores: enum_map! {
				Ability::Strength => 10,
				Ability::Dexterity => 10,
				Ability::Constitution => 10,
				Ability::Intelligence => 10,
				Ability::Wisdom => 10,
				Ability::Charisma => 10,
			},
			culture: changeling_culture(),
			selected_values: HashMap::from([
				(
					PathBuf::from("culture\\Incognito\\AbilityScoreIncrease"),
					"con".into(),
				),
				(
					PathBuf::from("culture\\Incognito\\GoodWithPeople"),
					"Insight".into(),
				),
				(
					PathBuf::from("culture\\Incognito\\Languages\\langA"),
					"Draconic".into(),
				),
				(
					PathBuf::from("culture\\Incognito\\Languages\\langB"),
					"Undercommon".into(),
				),
			]),
		};
		assert_eq!(
			character.compile_stats(),
			CompiledStats {
				ability_scores: enum_map! {
					Ability::Strength => 10,
					Ability::Dexterity => 10,
					Ability::Constitution => 11,
					Ability::Intelligence => 10,
					Ability::Wisdom => 10,
					Ability::Charisma => 12,
				},
				skills: enum_map! {
					Skill::Acrobatics => ProficiencyLevel::None,
					Skill::AnimalHandling => ProficiencyLevel::None,
					Skill::Arcana => ProficiencyLevel::None,
					Skill::Athletics => ProficiencyLevel::None,
					Skill::Deception => ProficiencyLevel::None,
					Skill::History => ProficiencyLevel::None,
					Skill::Insight => ProficiencyLevel::Full,
					Skill::Intimidation => ProficiencyLevel::None,
					Skill::Investigation => ProficiencyLevel::None,
					Skill::Medicine => ProficiencyLevel::None,
					Skill::Nature => ProficiencyLevel::None,
					Skill::Perception => ProficiencyLevel::None,
					Skill::Performance => ProficiencyLevel::None,
					Skill::Persuasion => ProficiencyLevel::None,
					Skill::Religion => ProficiencyLevel::None,
					Skill::SleightOfHand => ProficiencyLevel::None,
					Skill::Stealth => ProficiencyLevel::None,
					Skill::Survival => ProficiencyLevel::None,
				},
				languages: vec!["Common".into(), "Draconic".into(), "Undercommon".into()],
				life_expectancy: 100,
				max_height: (
					60,
					RollSet(enum_map! {
						Die::D4 => 2,
						Die::D6 => 0,
						Die::D8 => 0,
						Die::D10 => 0,
						Die::D12 => 0,
						Die::D20 => 0,
					})
				),
				missing_selections: vec![],
			}
		);
	}
}
