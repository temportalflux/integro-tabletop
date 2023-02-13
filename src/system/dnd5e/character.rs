use super::{
	condition::BoxedCondition,
	modifier::{BoxedModifier, Container},
	roll::RollSet,
	Ability, Feature, ProficiencyLevel, Score, Skill,
};
use enum_map::EnumMap;
use std::{
	collections::{BTreeMap, BTreeSet, HashMap},
	path::PathBuf,
	rc::Rc,
};

mod background;
pub use background::*;
mod class;
pub use class::*;
mod description;
pub use description::*;
mod lineage;
pub use lineage::*;
mod upbringing;
pub use upbringing::*;
pub mod inventory;

/// Core character data which is (de)serializable and
/// from which the derived data can be compiled.
#[derive(Clone, PartialEq)]
pub struct Character {
	pub lineages: [Option<Lineage>; 2],
	pub upbringing: Option<Upbringing>,
	pub background: Option<Background>,
	pub classes: Vec<Class>,
	pub feats: Vec<Feature>,
	pub description: Description,
	pub ability_scores: EnumMap<Ability, Score>,
	pub selected_values: HashMap<PathBuf, String>,
	pub inventory: inventory::Inventory,
	pub conditions: Vec<BoxedCondition>,
	pub hit_points: (u32, u32),
}
impl Character {
	pub fn with_culture(mut self, culture: Culture) -> Self {
		let [a, b] = culture.lineages;
		self.lineages = [Some(a), Some(b)];
		self.upbringing = Some(culture.upbringing);
		self
	}

	pub fn compile(&self) -> Derived {
		let mut stats = DerivedBuilder::new(self);

		for lineage in &self.lineages {
			if let Some(lineage) = lineage {
				stats.apply_from(lineage);
			}
		}
		if let Some(upbringing) = &self.upbringing {
			stats.apply_from(upbringing);
		}
		if let Some(background) = &self.background {
			stats.apply_from(background);
		}
		for class in &self.classes {
			stats.apply_from(class);
		}
		for feat in &self.feats {
			stats.apply_from(feat);
		}

		stats.build()
	}

	pub fn level(&self) -> i32 {
		self.classes.iter().map(|class| class.level_count()).sum()
	}
}

/// Data derived from the `Character`, such as bonuses to abilities/skills,
/// proficiencies, and actions. This data all lives within `Character` in
/// its various features and subtraits, and is compiled into one flat
/// structure for easy reference when displaying the character information.
#[derive(Clone, Default, PartialEq)]
pub struct Derived {
	missing_selections: Vec<PathBuf>,
	ability_scores: EnumMap<Ability, AttributedValue<i32>>,
	saving_throws: EnumMap<
		Ability,
		(
			/*is proficient*/ AttributedValue<ProficiencyLevel>,
			/*adv modifiers*/ Vec<(String, PathBuf)>,
		),
	>,
	skills: EnumMap<Skill, AttributedValue<ProficiencyLevel>>,
	languages: BTreeMap<String, BTreeSet<PathBuf>>,
	pub life_expectancy: i32,
	pub max_height: (i32, RollSet),
	max_hit_points: u32,
}

impl std::fmt::Debug for Derived {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Derived {{\
			\n\tmissing_selections: {:?}\
			\n\tability_scores: {}\
			\n\tskills: {}\
			\n\tlanguages: {}\
			\n\tlife_expectancy: {:?}\
			\n\tmax_height: {:?}\
			\n}}",
			self.missing_selections,
			self.ability_scores
				.iter()
				.fold(String::new(), |str, (ability, attributed)| {
					let sources = attributed
						.sources
						.iter()
						.fold(String::new(), |str, (src_path, value)| {
							format!("{str}\n\t\t\t{src_path:?}: {value:?}")
						});
					format!("{str}\n\t\t{ability:?}: {:?}{sources}", attributed.value)
				}),
			self.skills
				.iter()
				.fold(String::new(), |str, (skill, attributed)| {
					let sources = attributed
						.sources
						.iter()
						.fold(String::new(), |str, (src_path, value)| {
							format!("{str}\n\t\t\t{src_path:?}: {value:?}")
						});
					format!("{str}\n\t\t{skill:?}: {:?}{sources}", attributed.value)
				}),
			self.languages
				.iter()
				.fold(String::new(), |str, (lang, sources)| {
					format!("{str}\n\t\t{lang:?} => {sources:?}")
				}),
			self.life_expectancy,
			self.max_height,
		)
	}
}

/// The builder which compiles `Derived` from `Character`.
pub struct DerivedBuilder<'c> {
	character: &'c Character,
	derived: Derived,
	scope: PathBuf,
}
impl<'c> DerivedBuilder<'c> {
	pub fn new(character: &'c Character) -> Self {
		Self {
			character,
			derived: Derived::default(),
			scope: PathBuf::new(),
		}
	}

	pub fn scope(&self) -> PathBuf {
		match std::path::MAIN_SEPARATOR {
			'/' => self.scope.clone(),
			_ => PathBuf::from(
				self.scope
					.iter()
					.map(|s| s.to_str().unwrap())
					.collect::<Vec<_>>()
					.join("/"),
			),
		}
	}

	pub fn apply_from(&mut self, modifiers: &impl Container) {
		self.scope.push(&modifiers.id());
		modifiers.apply_modifiers(self);
		self.scope.pop();
	}

	pub fn apply(&mut self, modifier: &BoxedModifier) {
		let id = modifier.scope_id();
		if let Some(id) = id.as_ref() {
			self.scope.push(*id);
		}
		modifier.apply(self);
		if id.is_some() {
			self.scope.pop();
		}
	}

	pub fn get_selection(&mut self) -> Option<&str> {
		let selection = self
			.character
			.selected_values
			.get(&self.scope())
			.map(String::as_str);
		if selection.is_none() {
			self.derived.missing_selections.push(self.scope());
		}
		selection
	}

	pub fn build(self) -> Derived {
		self.derived
	}

	pub fn add_to_ability_score(&mut self, ability: Ability, bonus: i32) {
		let scope = self.scope();
		self.derived.ability_scores[ability].push(bonus, scope);
	}

	pub fn add_skill(&mut self, skill: Skill, proficiency: ProficiencyLevel) {
		let scope = self.scope();
		self.derived.skills[skill].push(proficiency, scope);
	}

	pub fn add_saving_throw(&mut self, ability: Ability) {
		let scope = self.scope();
		self.derived.saving_throws[ability]
			.0
			.push(ProficiencyLevel::Full, scope);
	}

	pub fn add_saving_throw_modifier(&mut self, ability: Ability, target: String) {
		let scope = self.scope();
		self.derived.saving_throws[ability].1.push((target, scope));
	}

	pub fn add_language(&mut self, language: String) {
		let scope = self.scope();
		match self.derived.languages.get_mut(&language) {
			Some(sources) => {
				sources.insert(scope);
			}
			None => {
				self.derived
					.languages
					.insert(language.clone(), BTreeSet::from([scope]));
			}
		}
	}
}
impl<'c> std::ops::Deref for DerivedBuilder<'c> {
	type Target = Derived;

	fn deref(&self) -> &Self::Target {
		&self.derived
	}
}
impl<'c> std::ops::DerefMut for DerivedBuilder<'c> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.derived
	}
}

/// The pairing of `Character` and `Derived` to form a singlular reference
/// structure for all character data.
#[derive(Clone, PartialEq)]
pub struct State {
	character: Character,
	derived: Derived,
}
impl From<Character> for State {
	fn from(character: Character) -> Self {
		let derived = character.compile();
		Self { character, derived }
	}
}
impl yew::Reducible for State {
	type Action = yew::Callback<Self, Self>;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		Rc::new(action.emit((*self).clone()))
	}
}
impl State {
	/// Returns the score/value for a given ability. Any bonuses beyond the character's base scores
	/// are provided with a path to the feature which provided that bonus.
	pub fn ability_score(&self, ability: Ability) -> (Score, Vec<(PathBuf, i32)>) {
		let mut score = self.character.ability_scores[ability];
		let original_score = score.0;
		let attributed = &self.derived.ability_scores[ability];
		(*score) += attributed.value;
		let mut sources = attributed.sources.clone();
		sources.insert(0, ("".into(), original_score));
		(score, sources)
	}

	pub fn proficiency_bonus(&self) -> i32 {
		match self.character.level().abs() {
			1..=4 => 2,
			5..=8 => 3,
			9..=12 => 4,
			13..=16 => 5,
			17.. => 6,
			_ => 0,
		}
	}

	pub fn initiative_bonus(&self) -> i32 {
		self.ability_score(Ability::Dexterity).0.modifier()
	}

	pub fn armor_class(&self) -> i32 {
		10 + self.ability_score(Ability::Dexterity).0.modifier()
	}

	pub fn ability_modifier(&self, ability: Ability, proficiency: ProficiencyLevel) -> i32 {
		let modifier = self.ability_score(ability).0.modifier();
		let prof_bonus_multiplier = match proficiency {
			ProficiencyLevel::None => 0.0,
			ProficiencyLevel::Half => 0.5,
			ProficiencyLevel::Full => 1.0,
			ProficiencyLevel::Double => 2.0,
		};
		let bonus = ((self.proficiency_bonus() as f32) * prof_bonus_multiplier).floor() as i32;
		modifier + bonus
	}

	pub fn saving_throw(&self, ability: Ability) -> &AttributedValue<ProficiencyLevel> {
		&self.derived.saving_throws[ability].0
	}

	pub fn saving_throw_modifiers(&self) -> EnumMap<Ability, Option<&Vec<(String, PathBuf)>>> {
		let mut values = EnumMap::default();
		for (ability, (_, modifiers)) in &self.derived.saving_throws {
			values[ability] = Some(modifiers);
		}
		values
	}

	/// Returns attributed skill proficiencies for the character.
	pub fn get_skill(&self, skill: Skill) -> &AttributedValue<ProficiencyLevel> {
		&self.derived.skills[skill]
	}

	pub fn languages(&self) -> &BTreeMap<String, BTreeSet<PathBuf>> {
		&self.derived.languages
	}

	pub fn hit_points(&self) -> (u32, u32, u32) {
		(
			self.character.hit_points.0,
			self.derived.max_hit_points,
			self.character.hit_points.1,
		)
	}

	pub fn add_hit_points(&mut self, amt: u32) {
		self.character.hit_points.0 = self.character.hit_points.0.saturating_add(amt);
	}

	pub fn sub_hit_points(&mut self, amt: u32) {
		self.character.hit_points.0 = self.character.hit_points.0.saturating_sub(amt);
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AttributedValue<T> {
	value: T,
	sources: Vec<(PathBuf, T)>,
}
impl<T> AttributedValue<T>
where
	T: Clone,
{
	pub fn set(&mut self, value: T, source: PathBuf) {
		self.value = value.clone();
		self.sources.push((source, value));
	}

	pub fn push(&mut self, value: T, source: PathBuf)
	where
		T: PartialOrd,
	{
		if self.value < value {
			self.value = value.clone();
		}
		self.sources.push((source, value));
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn sources(&self) -> &Vec<(PathBuf, T)> {
		&self.sources
	}
}

#[derive(Clone, PartialEq)]
pub struct Culture {
	pub lineages: [Lineage; 2],
	pub upbringing: Upbringing,
}

pub fn changeling_character() -> Character {
	use crate::system::dnd5e::hardcoded::*;
	use enum_map::enum_map;
	let culture = Culture {
		lineages: [changeling1(), changeling2()],
		upbringing: incognito(),
	};
	let background = anthropologist();
	Character {
		description: Description {
			name: "changeling".into(),
			pronouns: "".into(),
		},
		ability_scores: enum_map! {
			Ability::Strength => Score(10),
			Ability::Dexterity => Score(10),
			Ability::Constitution => Score(10),
			Ability::Intelligence => Score(10),
			Ability::Wisdom => Score(10),
			Ability::Charisma => Score(10),
		},
		lineages: [None, None],
		upbringing: None,
		background: Some(background),
		classes: Vec::new(),
		feats: Vec::new(),
		selected_values: HashMap::from([
			(
				PathBuf::from("Incognito/AbilityScoreIncrease"),
				"con".into(),
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
		inventory: inventory::Inventory::new(),
		conditions: Vec::new(),
		hit_points: (0, 0),
	}
	.with_culture(culture)
}

#[cfg(test)]
mod test {
	use super::*;
	use enum_map::enum_map;

	#[test]
	fn test_changeling() {
		use super::super::roll::Die;
		let character = changeling_character();
		assert_eq!(
			character.compile(),
			Derived {
				max_hit_points: 0,
				ability_scores: enum_map! {
					Ability::Strength => AttributedValue { value: 0, sources: vec![] },
					Ability::Dexterity => AttributedValue { value: 0, sources: vec![] },
					Ability::Constitution => AttributedValue { value: 1, sources: vec![
						(PathBuf::from("Incognito/AbilityScoreIncrease"), 1),
					] },
					Ability::Intelligence => AttributedValue { value: 0, sources: vec![] },
					Ability::Wisdom => AttributedValue { value: 0, sources: vec![] },
					Ability::Charisma => AttributedValue { value: 2, sources: vec![
						(PathBuf::from("Incognito/AbilityScoreIncrease"), 2),
					] },
				},
				saving_throws: enum_map! {
					Ability::Strength => (AttributedValue { value: ProficiencyLevel::None, sources: vec![] }, Vec::new()),
					Ability::Dexterity => (AttributedValue { value: ProficiencyLevel::None, sources: vec![] }, Vec::new()),
					Ability::Constitution => (AttributedValue { value: ProficiencyLevel::None, sources: vec![] }, Vec::new()),
					Ability::Intelligence => (AttributedValue { value: ProficiencyLevel::None, sources: vec![] }, Vec::new()),
					Ability::Wisdom => (AttributedValue { value: ProficiencyLevel::None, sources: vec![] }, Vec::new()),
					Ability::Charisma => (AttributedValue { value: ProficiencyLevel::None, sources: vec![] }, Vec::new()),
				},
				skills: enum_map! {
					Skill::Acrobatics => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::AnimalHandling => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Arcana => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Athletics => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Deception => AttributedValue { value: ProficiencyLevel::Full, sources: vec![
						(PathBuf::from("Incognito/GoodWithPeople"), ProficiencyLevel::Full),
					] },
					Skill::History => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Insight => AttributedValue { value: ProficiencyLevel::Full, sources: vec![
						(PathBuf::from("Anthropologist/SkillProficiencies"), ProficiencyLevel::Full),
					] },
					Skill::Intimidation => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Investigation => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Medicine => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Nature => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Perception => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Performance => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Persuasion => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Religion => AttributedValue { value: ProficiencyLevel::Full, sources: vec![
						(PathBuf::from("Anthropologist/SkillProficiencies"), ProficiencyLevel::Full),
					] },
					Skill::SleightOfHand => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Stealth => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Survival => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
				},
				languages: BTreeMap::from([
					(
						"Common".into(),
						BTreeSet::from([PathBuf::from("Incognito/Languages")])
					),
					(
						"Draconic".into(),
						BTreeSet::from([PathBuf::from("Incognito/Languages/langA")])
					),
					(
						"Undercommon".into(),
						BTreeSet::from([PathBuf::from("Incognito/Languages/langB")])
					),
					(
						"Sylvan".into(),
						BTreeSet::from([PathBuf::from("Anthropologist/Languages/langA")])
					),
					(
						"Elvish".into(),
						BTreeSet::from([PathBuf::from("Anthropologist/Languages/langB")])
					),
				]),
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
