use crate::path_map::PathMap;

use super::{
	condition::BoxedCondition,
	modifier::{BoxedModifier, Container, Defense},
	proficiency,
	roll::RollSet,
	Ability, BoxedFeature, Score, Skill,
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
	pub feats: Vec<BoxedFeature>,
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
			stats.add_feature(feat);
		}
		stats.apply_from(&self.inventory);

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
			/*is proficient*/ AttributedValue<proficiency::Level>,
			/*adv modifiers*/ Vec<(String, PathBuf)>,
		),
	>,
	skills: EnumMap<
		Skill,
		(
			/*proficiency*/ AttributedValue<proficiency::Level>,
			/*modifiers*/
			EnumMap<super::roll::Modifier, Vec<(/*context*/ Option<String>, /*source*/ PathBuf)>>,
		),
	>,
	proficiencies: EnumMap<proficiency::Kind, BTreeMap<String, BTreeSet<PathBuf>>>,
	speeds: BTreeMap<String, AttributedValue<i32>>,
	senses: BTreeMap<String, AttributedValue<i32>>,
	defenses: EnumMap<Defense, BTreeMap<String, BTreeSet<PathBuf>>>,
	features: PathMap<BoxedFeature>,
	pub life_expectancy: i32,
	pub max_height: (i32, RollSet),
	max_hit_points: u32,
}

impl std::fmt::Debug for Derived {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Derived {{\
			\n  missing_selections: {:?}\
			\n  ability_scores: {}\
			\n  skills: {}\
			\n  proficiencies: {}\
			\n  speeds: {}\
			\n  senses: {}\
			\n  defenses: {}\
			\n  features: {}\
			\n  life_expectancy: {:?}\
			\n  max_height: {:?}\
			\n}}",
			self.missing_selections,
			self.ability_scores
				.iter()
				.fold(String::new(), |str, (ability, attributed)| {
					format!(
						"{str}\n    {ability:?}: {:?}{}",
						attributed.value,
						attributed
							.sources
							.iter()
							.fold(String::new(), |str, (src_path, value)| {
								format!("{str}\n      {src_path:?}: {value:?}")
							})
					)
				}),
			/*
				"Perception": Full
					"Lineage1/FeatureA": Full
					"Advantage":
						"Lineage2/FeatureC": "when using hearing or smell"
			*/
			self.skills
				.iter()
				.fold(String::new(), |str, (skill, (attributed, modifiers))| {
					let prof = attributed.value;
					let sources = attributed
						.sources
						.iter()
						.fold(String::new(), |str, (src_path, value)| {
							format!("{str}\n      {src_path:?}: {value:?}")
						});
					let modifiers =
						modifiers
							.iter()
							.fold(String::new(), |str, (modifier, contexts)| {
								let sources =
									contexts.iter().fold(String::new(), |str, (ctx, source)| {
										format!("{str}\n        {source:?}: {ctx:?}")
									});
								format!("{str}\n      {modifier:?}:{sources}")
							});
					format!("{str}\n    {skill:?}: {prof:?}{sources}{modifiers}")
				}),
			self.proficiencies
				.iter()
				.fold(String::new(), |str, (kind, attributed_values)| {
					format!(
						"{str}\n    {kind:?} => {}",
						attributed_values
							.iter()
							.fold(String::new(), |str, (value, sources)| {
								format!("{str}\n      {value:?} => {sources:?}")
							})
					)
				}),
			self.speeds
				.iter()
				.fold(String::new(), |str, (kind, attributed)| {
					format!(
						"{str}\n    {kind:?}: {:?}{}",
						attributed.value,
						attributed
							.sources
							.iter()
							.fold(String::new(), |str, (src_path, value)| {
								format!("{str}\n      {src_path:?}: {value:?}")
							})
					)
				}),
			self.senses
				.iter()
				.fold(String::new(), |str, (kind, attributed)| {
					format!(
						"{str}\n    {kind:?}: {:?}{}",
						attributed.value,
						attributed
							.sources
							.iter()
							.fold(String::new(), |str, (src_path, value)| {
								format!("{str}\n      {src_path:?}: {value:?}")
							})
					)
				}),
			self.defenses
				.iter()
				.fold(String::new(), |str, (kind, attributed_values)| {
					format!(
						"{str}\n    {kind:?} => {}",
						attributed_values
							.iter()
							.fold(String::new(), |str, (value, sources)| {
								format!("{str}\n      {value:?} => {sources:?}")
							})
					)
				}),
			self.features
				.as_vec()
				.into_iter()
				.fold(String::new(), |str, (path, feat)| {
					format!("{str}\n    {path:?}: {}", feat.inner().name)
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
		let id = modifiers.id();
		if let Some(id) = &id {
			self.scope.push(id);
		}
		modifiers.apply_modifiers(self);
		if id.is_some() {
			self.scope.pop();
		}
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

	pub fn add_feature(&mut self, feature: &BoxedFeature) {
		let scope = self.scope();
		self.features.insert(&scope, feature.clone());
		self.apply_from(feature.inner());
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

	pub fn add_skill(&mut self, skill: Skill, proficiency: proficiency::Level) {
		let scope = self.scope();
		self.derived.skills[skill].0.push(proficiency, scope);
	}

	pub fn add_skill_modifier(
		&mut self,
		skill: Skill,
		modifier: super::roll::Modifier,
		context: Option<String>,
	) {
		let scope = self.scope();
		self.derived.skills[skill].1[modifier].push((context, scope));
	}

	pub fn add_saving_throw(&mut self, ability: Ability) {
		let scope = self.scope();
		self.derived.saving_throws[ability]
			.0
			.push(proficiency::Level::Full, scope);
	}

	pub fn add_saving_throw_modifier(&mut self, ability: Ability, target: String) {
		let scope = self.scope();
		self.derived.saving_throws[ability].1.push((target, scope));
	}

	pub fn add_language(&mut self, language: String) {
		let scope = self.scope();
		match self.derived.proficiencies[proficiency::Kind::Language].get_mut(&language) {
			Some(sources) => {
				sources.insert(scope);
			}
			None => {
				self.derived.proficiencies[proficiency::Kind::Language]
					.insert(language, BTreeSet::from([scope]));
			}
		}
	}

	pub fn add_max_speed(&mut self, kind: String, max_bound_in_feet: i32) {
		let scope = self.scope();
		match self.derived.speeds.get_mut(&kind) {
			Some(value) => {
				value.push(max_bound_in_feet, scope);
			}
			None => {
				let mut value = AttributedValue::default();
				value.push(max_bound_in_feet, scope);
				self.derived.speeds.insert(kind, value);
			}
		}
	}

	pub fn add_max_sense(&mut self, kind: String, max_bound_in_feet: i32) {
		let scope = self.scope();
		match self.derived.senses.get_mut(&kind) {
			Some(value) => {
				value.push(max_bound_in_feet, scope);
			}
			None => {
				let mut value = AttributedValue::default();
				value.push(max_bound_in_feet, scope);
				self.derived.senses.insert(kind.clone(), value);
			}
		}
	}

	pub fn add_defense(&mut self, kind: Defense, target: String) {
		let scope = self.scope();
		match self.derived.defenses[kind].get_mut(&target) {
			Some(sources) => {
				sources.insert(scope);
			}
			None => {
				self.derived.defenses[kind].insert(target, BTreeSet::from([scope]));
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
	pub fn recompile(&mut self) {
		self.derived = self.character.compile();
	}

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

	pub fn ability_modifier(&self, ability: Ability, proficiency: proficiency::Level) -> i32 {
		let modifier = self.ability_score(ability).0.modifier();
		let prof_bonus_multiplier = match proficiency {
			proficiency::Level::None => 0.0,
			proficiency::Level::Half => 0.5,
			proficiency::Level::Full => 1.0,
			proficiency::Level::Double => 2.0,
		};
		let bonus = ((self.proficiency_bonus() as f32) * prof_bonus_multiplier).floor() as i32;
		modifier + bonus
	}

	pub fn saving_throw(&self, ability: Ability) -> &AttributedValue<proficiency::Level> {
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
	pub fn get_skill(
		&self,
		skill: Skill,
	) -> &(
		/*proficiency*/ AttributedValue<proficiency::Level>,
		/*modifiers*/
		EnumMap<super::roll::Modifier, Vec<(/*context*/ Option<String>, /*source*/ PathBuf)>>,
	) {
		&self.derived.skills[skill]
	}

	pub fn get_proficiencies(
		&self,
		kind: proficiency::Kind,
	) -> &BTreeMap<String, BTreeSet<PathBuf>> {
		&self.derived.proficiencies[kind]
	}

	pub fn speeds(&self) -> &BTreeMap<String, AttributedValue<i32>> {
		&self.derived.speeds
	}

	pub fn senses(&self) -> &BTreeMap<String, AttributedValue<i32>> {
		&self.derived.senses
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

	pub fn defenses(&self) -> &EnumMap<Defense, BTreeMap<String, BTreeSet<PathBuf>>> {
		&self.derived.defenses
	}

	pub fn inventory(&self) -> &inventory::Inventory {
		&self.character.inventory
	}

	pub fn inventory_mut(&mut self) -> &mut inventory::Inventory {
		&mut self.character.inventory
	}

	pub fn features(&self) -> &PathMap<BoxedFeature> {
		&self.derived.features
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
		use super::super::roll::*;
		let character = changeling_character();
		let none_skill = (
			AttributedValue {
				value: proficiency::Level::None,
				sources: vec![],
			},
			enum_map! {
				Modifier::Advantage => vec![],
				Modifier::Disadvantage => vec![],
			},
		);
		let derived = character.compile();
		assert_eq!(derived.max_hit_points, 0);
		assert_eq!(
			derived.ability_scores,
			enum_map! {
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
			}
		);
		assert_eq!(
			derived.saving_throws,
			enum_map! {
				Ability::Strength => (AttributedValue { value: proficiency::Level::None, sources: vec![] }, Vec::new()),
				Ability::Dexterity => (AttributedValue { value: proficiency::Level::None, sources: vec![] }, Vec::new()),
				Ability::Constitution => (AttributedValue { value: proficiency::Level::None, sources: vec![] }, Vec::new()),
				Ability::Intelligence => (AttributedValue { value: proficiency::Level::None, sources: vec![] }, Vec::new()),
				Ability::Wisdom => (AttributedValue { value: proficiency::Level::None, sources: vec![] }, Vec::new()),
				Ability::Charisma => (AttributedValue { value: proficiency::Level::None, sources: vec![] }, Vec::new()),
			}
		);
		assert_eq!(
			derived.skills,
			enum_map! {
				Skill::Acrobatics => none_skill.clone(),
				Skill::AnimalHandling => none_skill.clone(),
				Skill::Arcana => none_skill.clone(),
				Skill::Athletics => none_skill.clone(),
				Skill::Deception => (
					AttributedValue { value: proficiency::Level::Full, sources: vec![
						(PathBuf::from("Incognito/GoodWithPeople"), proficiency::Level::Full),
					] },
					enum_map! {
						Modifier::Advantage => vec![],
						Modifier::Disadvantage => vec![],
					}
				),
				Skill::History => none_skill.clone(),
				Skill::Insight => (
					AttributedValue { value: proficiency::Level::Full, sources: vec![
						(PathBuf::from("Anthropologist/SkillProficiencies"), proficiency::Level::Full),
					] },
					enum_map! {
						Modifier::Advantage => vec![],
						Modifier::Disadvantage => vec![],
					}
				),
				Skill::Intimidation => none_skill.clone(),
				Skill::Investigation => none_skill.clone(),
				Skill::Medicine => none_skill.clone(),
				Skill::Nature => none_skill.clone(),
				Skill::Perception => none_skill.clone(),
				Skill::Performance => none_skill.clone(),
				Skill::Persuasion => none_skill.clone(),
				Skill::Religion => (
					AttributedValue { value: proficiency::Level::Full, sources: vec![
						(PathBuf::from("Anthropologist/SkillProficiencies"), proficiency::Level::Full),
					] },
					enum_map! {
						Modifier::Advantage => vec![],
						Modifier::Disadvantage => vec![],
					}
				),
				Skill::SleightOfHand => none_skill.clone(),
				Skill::Stealth => none_skill.clone(),
				Skill::Survival => none_skill.clone(),
			}
		);
		assert_eq!(
			derived.proficiencies,
			enum_map! {
				proficiency::Kind::Language => BTreeMap::from([
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
				proficiency::Kind::Armor => BTreeMap::from([]),
				proficiency::Kind::Weapon => BTreeMap::from([]),
				proficiency::Kind::Tool => BTreeMap::from([]),
			}
		);
		assert_eq!(
			derived.speeds,
			BTreeMap::from([(
				"Walking".into(),
				AttributedValue {
					value: 30,
					sources: vec![(PathBuf::from("ChangelingI/Speeds"), 30),]
				}
			)])
		);
		assert_eq!(derived.senses, BTreeMap::from([]));
		assert_eq!(
			derived.defenses,
			enum_map! {
				Defense::Resistant => BTreeMap::from([]),
				Defense::Immune =>BTreeMap::from([]),
				Defense::Vulnerable =>BTreeMap::from([]),
			}
		);
		//assert_eq!(derived.features, BTreeMap::from([]));
		assert_eq!(derived.life_expectancy, 100);
		assert_eq!(
			derived.max_height,
			(
				60,
				RollSet(enum_map! {
					Die::D4 => 2,
					Die::D6 => 0,
					Die::D8 => 0,
					Die::D10 => 0,
					Die::D12 => 0,
					Die::D20 => 0,
				})
			)
		);
		assert_eq!(derived.missing_selections, Vec::<PathBuf>::new());
	}
}
