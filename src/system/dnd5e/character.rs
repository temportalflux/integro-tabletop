use super::{
	condition::BoxedCondition,
	modifier::{
		AddAbilityScore, AddLanguage, AddLifeExpectancy, AddMaxHeight, AddSkill, BoxedModifier,
		Container, Selector,
	},
	roll::{Die, Roll, RollSet},
	Ability, Action, Feature, ProficiencyLevel, Score, Skill,
};
use enum_map::EnumMap;
use enumset::EnumSet;
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
	lineages: [Option<Lineage>; 2],
	upbringing: Option<Upbringing>,
	background: Option<Background>,
	classes: Vec<Class>,
	#[allow(dead_code)]
	description: Description,
	ability_scores: EnumMap<Ability, Score>,
	selected_values: HashMap<PathBuf, String>,
	inventory: inventory::Inventory,
	conditions: Vec<BoxedCondition>,
	hit_points: (u32, u32),
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
	pub fn ability_score(&self, ability: Ability) -> (Score, &Vec<(PathBuf, i32)>) {
		let mut score = self.character.ability_scores[ability];
		let attributed = &self.derived.ability_scores[ability];
		(*score) += attributed.value;
		(score, &attributed.sources)
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

	/// Returns attributed skill proficiencies for the character.
	pub fn get_skills(&self) -> &EnumMap<Skill, AttributedValue<ProficiencyLevel>> {
		&self.derived.skills
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
}

#[derive(Clone, PartialEq)]
pub struct Culture {
	pub lineages: [Lineage; 2],
	pub upbringing: Upbringing,
}

pub fn changeling_character() -> Character {
	use enum_map::enum_map;
	let culture = {
		let life_expectancy = Feature {
			name: "Age".into(),
			description: "Your life expectancy increases by 50 years.".into(),
			modifiers: vec![AddLifeExpectancy(50).into()],
			..Default::default()
		};
		let size = Feature {
			name: "Size".into(),
			description: "Your height increases by 30 + 1d4 inches.".into(),
			modifiers: vec![
				AddMaxHeight::Value(30).into(),
				AddMaxHeight::Roll(Roll {
					amount: 1,
					die: Die::D4,
				})
				.into(),
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
							AddAbilityScore {
								ability: Selector::Specific(Ability::Charisma),
								value: 2,
							}.into(),
							AddAbilityScore {
								ability: Selector::AnyOf {
									id: None,
									options: EnumSet::only(Ability::Charisma).complement().into_iter().collect(),
								},
								value: 1,
							}.into(),
						],
						..Default::default()
					},
					Feature {
						name: "Good with People".into(),
						description: "You gain proficiency with two of the following skills of your choice: Deception, Insight, Intimidation, and Persuasion.".into(),
						modifiers: vec![
							AddSkill {
								skill: Selector::AnyOf {
									id: None,
									options: vec![
										Skill::Deception, Skill::Insight, Skill::Intimidation, Skill::Persuasion,
									],
								},
								proficiency: ProficiencyLevel::Full,
							}.into(),
						],
						..Default::default()
					},
					Feature {
						name: "Languages".into(),
						description: "You can speak, read, and write Common and two other languages of your choice.".into(),
						modifiers: vec![
							AddLanguage(Selector::Specific("Common".into())).into(),
							AddLanguage(Selector::Any { id: Some("langA".into()) }).into(),
							AddLanguage(Selector::Any { id: Some("langB".into()) }).into(),
						],
						..Default::default()
					},
				],
			},
		}
	};
	let background = Background {
		name: "Anthropologist".into(),
		description: "You have always been fascinated by other cultures, from the most ancient and \
		primeval lost lands to the most modern civilizations. By studying other cultures' customs, philosophies, \
		laws, rituals, religious beliefs, languages, and art, you have learned how tribes, empires, \
		and all forms of society in between craft their own destinies and doom. This knowledge came to \
		you not only through books and scrolls, but also through first-hand observation—by visiting far-flung \
		settlements and exploring local histories and customs.".into(),
		features: vec![
			Feature {
				name: "Skill Proficiencies".into(),
				modifiers: vec![
					AddSkill {
						skill: Selector::Specific(Skill::Insight),
						proficiency: ProficiencyLevel::Full,
					}.into(),
					AddSkill {
						skill: Selector::Specific(Skill::Religion),
						proficiency: ProficiencyLevel::Full,
					}.into(),
				],
				..Default::default()
			},
			Feature {
				name: "Languages".into(),
				modifiers: vec![
					AddLanguage(Selector::Any { id: Some("langA".into()) }).into(),
					AddLanguage(Selector::Any { id: Some("langB".into()) }).into(),
				],
				..Default::default()
			},
			Feature {
				name: "Adept Linguist".into(),
				description: "You can communicate with humanoids who don't speak any language you know. \
				You must observe the humanoids interacting with one another for at least 1 day, \
				after which you learn a handful of important words, expressions, and gestures—enough \
				to communicate on a rudimentary level.".into(),
				..Default::default()
			},
		],
	};
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
