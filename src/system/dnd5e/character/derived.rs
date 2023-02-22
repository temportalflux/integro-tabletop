use super::super::roll::Modifier;
use super::super::{
	action::Action, mutator::Defense, proficiency, roll::RollSet, Ability, BoxedFeature, Skill,
};
use super::*;
use crate::path_map::PathMap;
use enum_map::EnumMap;
use std::{
	collections::{BTreeMap, BTreeSet},
	path::PathBuf,
};

/// Data derived from the `Persistent`, such as bonuses to abilities/skills,
/// proficiencies, and actions. This data all lives within `Persistent` in
/// its various features and subtraits, and is compiled into one flat
/// structure for easy reference when displaying the character information.
#[derive(Clone, Default, PartialEq)]
pub struct Derived {
	pub missing_selections: Vec<PathBuf>,
	pub ability_scores: AbilityScores,
	pub saving_throws: SavingThrows,
	pub skills: Skills,
	pub other_proficiencies: OtherProficiencies,
	pub speeds: Speeds,
	pub senses: Senses,
	pub defenses: Defenses,
	pub features: PathMap<BoxedFeature>,
	pub max_hit_points: u32,
	pub armor_class: ArmorClass,
	pub actions: Vec<Action>,
	pub description: DerivedDescription,
}

#[derive(Clone, Default, PartialEq)]
pub struct AbilityScores(EnumMap<Ability, AttributedValue<i32>>);
impl AbilityScores {
	pub fn push_bonus(&mut self, ability: Ability, bonus: i32, source: PathBuf) {
		self.0[ability].push(bonus, source);
	}

	pub fn get(&self, ability: Ability) -> &AttributedValue<i32> {
		&self.0[ability]
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct SavingThrows(
	EnumMap<
		Ability,
		(
			/*is proficient*/ AttributedValue<proficiency::Level>,
			/*adv modifiers*/ Vec<(Option<String>, PathBuf)>,
		),
	>,
);
impl SavingThrows {
	pub fn add_proficiency(&mut self, ability: Ability, source: PathBuf) {
		self.0[ability].0.push(proficiency::Level::Full, source);
	}

	pub fn add_modifier(&mut self, ability: Ability, target: Option<String>, source: PathBuf) {
		self.0[ability].1.push((target, source));
	}

	pub fn iter_modifiers(
		&self,
	) -> impl Iterator<Item = (Ability, &Vec<(Option<String>, PathBuf)>)> {
		self.0
			.iter()
			.map(|(ability, (_, modifiers))| (ability, modifiers))
	}
}
impl std::ops::Index<Ability> for SavingThrows {
	type Output = (
		AttributedValue<proficiency::Level>,
		Vec<(Option<String>, PathBuf)>,
	);
	fn index(&self, index: Ability) -> &Self::Output {
		&self.0[index]
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct Skills(
	EnumMap<
		Skill,
		(
			/*proficiency*/ AttributedValue<proficiency::Level>,
			/*modifiers*/
			EnumMap<Modifier, Vec<(/*context*/ Option<String>, /*source*/ PathBuf)>>,
		),
	>,
);
impl Skills {
	pub fn add_proficiency(&mut self, skill: Skill, level: proficiency::Level, source: PathBuf) {
		self.0[skill].0.push(level, source);
	}

	pub fn add_modifier(
		&mut self,
		skill: Skill,
		modifier: Modifier,
		context: Option<String>,
		source: PathBuf,
	) {
		self.0[skill].1[modifier].push((context, source));
	}
}
impl std::ops::Index<Skill> for Skills {
	type Output = (
		/*proficiency*/ AttributedValue<proficiency::Level>,
		/*modifiers*/
		EnumMap<Modifier, Vec<(/*context*/ Option<String>, /*source*/ PathBuf)>>,
	);

	fn index(&self, index: Skill) -> &Self::Output {
		&self.0[index]
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct Speeds(BTreeMap<String, AttributedValue<i32>>);
impl Speeds {
	pub fn push_max(&mut self, kind: String, max_bound_in_feet: i32, source: PathBuf) {
		match self.0.get_mut(&kind) {
			Some(value) => {
				value.push(max_bound_in_feet, source);
			}
			None => {
				let mut value = AttributedValue::default();
				value.push(max_bound_in_feet, source);
				self.0.insert(kind, value);
			}
		}
	}
}
impl std::ops::Deref for Speeds {
	type Target = BTreeMap<String, AttributedValue<i32>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct Senses(BTreeMap<String, AttributedValue<i32>>);
impl Senses {
	pub fn push_max(&mut self, kind: String, max_bound_in_feet: i32, source: PathBuf) {
		match self.0.get_mut(&kind) {
			Some(value) => {
				value.push(max_bound_in_feet, source);
			}
			None => {
				let mut value = AttributedValue::default();
				value.push(max_bound_in_feet, source);
				self.0.insert(kind, value);
			}
		}
	}
}
impl std::ops::Deref for Senses {
	type Target = BTreeMap<String, AttributedValue<i32>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct Defenses(EnumMap<Defense, BTreeMap<String, BTreeSet<PathBuf>>>);
impl Defenses {
	pub fn push(&mut self, kind: Defense, target: String, source: PathBuf) {
		match self.0[kind].get_mut(&target) {
			Some(sources) => {
				sources.insert(source);
			}
			None => {
				self.0[kind].insert(target, BTreeSet::from([source]));
			}
		}
	}
}
impl std::ops::Deref for Defenses {
	type Target = EnumMap<Defense, BTreeMap<String, BTreeSet<PathBuf>>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Default, PartialEq)]
pub struct DerivedDescription {
	pub life_expectancy: i32,
	pub max_height: (i32, RollSet),
}
