use super::{
	modifier::{
		AddAbilityScore, AddLanguage, AddLifeExpectancy, AddMaxHeight, AddSkill, Container,
		Modifier, Selector,
	},
	roll::{Die, Roll, RollSet},
	Ability, Action, Feature, ProficiencyLevel, Skill,
};
use enum_map::EnumMap;
use enumset::EnumSet;
use std::{
	collections::{BTreeMap, BTreeSet, HashMap},
	path::PathBuf,
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

#[derive(Clone)]
pub struct Character {
	#[allow(dead_code)]
	description: Description,
	ability_scores: EnumMap<Ability, i32>,
	lineages: [Option<Lineage>; 2],
	upbringing: Option<Upbringing>,
	background: Option<Background>,
	classes: Vec<Class>,
	selected_values: HashMap<PathBuf, String>,
}
impl Character {
	pub fn level(&self) -> i32 {
		self.classes.iter().map(|class| class.level_count()).sum()
	}
}

pub struct StatsBuilder<'c> {
	character: &'c Character,
	stats: CompiledStats,
	scope: PathBuf,
}
impl<'c> StatsBuilder<'c> {
	pub fn new(character: &'c Character) -> Self {
		let mut stats = CompiledStats::default();
		stats.ability_scores = character.ability_scores;
		Self {
			character,
			stats,
			scope: PathBuf::new(),
		}
	}

	pub fn apply_from(&mut self, modifiers: &impl Container) {
		self.scope.push(&modifiers.id());
		modifiers.apply_modifiers(self);
		self.scope.pop();
	}

	pub fn apply(&mut self, modifier: &Box<dyn Modifier>) {
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
			.get(&self.scope)
			.map(String::as_str);
		if selection.is_none() {
			self.stats.missing_selections.push(self.scope.clone());
		}
		selection
	}

	pub fn build(self) -> CompiledStats {
		self.stats
	}

	pub fn add_skill(&mut self, skill: Skill, proficiency: ProficiencyLevel) {
		self.stats.skills[skill].push(proficiency, self.scope.clone());
	}

	pub fn add_language(&mut self, language: String) {
		match self.stats.languages.get_mut(&language) {
			Some(sources) => {
				sources.insert(self.scope.clone());
			}
			None => {
				self.stats
					.languages
					.insert(language.clone(), BTreeSet::from([self.scope.clone()]));
			}
		}
	}
}
impl<'c> std::ops::Deref for StatsBuilder<'c> {
	type Target = CompiledStats;

	fn deref(&self) -> &Self::Target {
		&self.stats
	}
}
impl<'c> std::ops::DerefMut for StatsBuilder<'c> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.stats
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct CompiledStats {
	pub missing_selections: Vec<PathBuf>,
	pub ability_scores: EnumMap<Ability, i32>,
	pub skills: EnumMap<Skill, AttributedValue<ProficiencyLevel>>,
	pub languages: BTreeMap<String, BTreeSet<PathBuf>>,
	pub life_expectancy: i32,
	pub max_height: (i32, RollSet),
}
impl std::fmt::Debug for CompiledStats {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "CompiledStats {{\
			\n\tmissing_selections: {:?}\
			\n\tability_scores: {:?}\
			\n\tskills: {}\
			\n\tlanguages: {}\
			\n\tlife_expectancy: {:?}\
			\n\tmax_height: {:?}\
			\n}}",
			self.missing_selections,
			self.ability_scores,
			self.skills.iter().fold(String::new(), |str, (skill, attributed)| {
				let sources = attributed.sources.iter().fold(String::new(), |str, (src_path, value)| {
					format!("{str}\n\t\t\t{src_path:?}: {value:?}")
				});
				format!("{str}\n\t\t{skill:?}: {:?}{sources}", attributed.value)
			}),
			self.languages.iter().fold(String::new(), |str, (lang, sources)| {
				format!("{str}\n\t\t{lang:?} => {sources:?}")
			}),
			self.life_expectancy,
			self.max_height,
		)
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
}

impl Character {
	pub fn with_culture(mut self, culture: Culture) -> Self {
		let [a, b] = culture.lineages;
		self.lineages = [Some(a), Some(b)];
		self.upbringing = Some(culture.upbringing);
		self
	}

	pub fn compile_stats(&self) -> CompiledStats {
		let mut stats = StatsBuilder::new(self);

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
}

#[derive(Clone, PartialEq)]
pub struct Culture {
	pub lineages: [Lineage; 2],
	pub upbringing: Upbringing,
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
			lineages: [None, None],
			upbringing: None,
			background: Some(Background {
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
							Box::new(AddSkill {
								skill: Selector::Specific(Skill::Insight),
								proficiency: ProficiencyLevel::Full,
							}),
							Box::new(AddSkill {
								skill: Selector::Specific(Skill::Religion),
								proficiency: ProficiencyLevel::Full,
							}),
						],
						..Default::default()
					},
					Feature {
						name: "Languages".into(),
						modifiers: vec![
							Box::new(AddLanguage(Selector::Any { id: Some("langA".into()) })),
							Box::new(AddLanguage(Selector::Any { id: Some("langB".into()) })),
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
			}),
			classes: Vec::new(),
			selected_values: HashMap::from([
				(
					PathBuf::from("Incognito\\AbilityScoreIncrease"),
					"con".into(),
				),
				(
					PathBuf::from("Incognito\\GoodWithPeople"),
					"Deception".into(),
				),
				(
					PathBuf::from("Incognito\\Languages\\langA"),
					"Draconic".into(),
				),
				(
					PathBuf::from("Incognito\\Languages\\langB"),
					"Undercommon".into(),
				),
				(
					PathBuf::from("Anthropologist\\Languages\\langA"),
					"Sylvan".into()
				),
				(
					PathBuf::from("Anthropologist\\Languages\\langB"),
					"Elvish".into()
				),
			]),
		}
		.with_culture(changeling_culture());
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
					Skill::Acrobatics => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::AnimalHandling => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Arcana => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Athletics => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Deception => AttributedValue { value: ProficiencyLevel::Full, sources: vec![
						(PathBuf::from("Incognito\\GoodWithPeople"), ProficiencyLevel::Full),
					] },
					Skill::History => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Insight => AttributedValue { value: ProficiencyLevel::Full, sources: vec![
						(PathBuf::from("Anthropologist\\SkillProficiencies"), ProficiencyLevel::Full),
					] },
					Skill::Intimidation => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Investigation => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Medicine => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Nature => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Perception => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Performance => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Persuasion => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Religion => AttributedValue { value: ProficiencyLevel::Full, sources: vec![
						(PathBuf::from("Anthropologist\\SkillProficiencies"), ProficiencyLevel::Full),
					] },
					Skill::SleightOfHand => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Stealth => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
					Skill::Survival => AttributedValue { value: ProficiencyLevel::None, sources: vec![] },
				},
				languages: BTreeMap::from([
					(
						"Common".into(),
						BTreeSet::from([PathBuf::from("Incognito\\Languages")])
					),
					(
						"Draconic".into(),
						BTreeSet::from([PathBuf::from("Incognito\\Languages\\langA")])
					),
					(
						"Undercommon".into(),
						BTreeSet::from([PathBuf::from("Incognito\\Languages\\langB")])
					),
					(
						"Sylvan".into(),
						BTreeSet::from([PathBuf::from("Anthropologist\\Languages\\langA")])
					),
					(
						"Elvish".into(),
						BTreeSet::from([PathBuf::from("Anthropologist\\Languages\\langB")])
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
