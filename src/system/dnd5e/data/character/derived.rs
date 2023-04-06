use super::AttributedValue;
use crate::{
	path_map::PathMap,
	system::dnd5e::{
		data::{
			action::Action,
			mutator::{Defense, Flag},
			proficiency,
			roll::{Modifier, RollSet},
			Ability, ArmorClass, BoxedFeature, DamageType, OtherProficiencies, Skill,
		},
		Value,
	},
};
use enum_map::{enum_map, EnumMap};
use std::{collections::BTreeMap, path::PathBuf};

mod ability_score;
pub use ability_score::*;
mod sense;
pub use sense::*;
mod speed;
pub use speed::*;

/// Data derived from the `Persistent`, such as bonuses to abilities/skills,
/// proficiencies, and actions. This data all lives within `Persistent` in
/// its various features and subtraits, and is compiled into one flat
/// structure for easy reference when displaying the character information.
#[derive(Clone, PartialEq, Debug)]
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
	pub max_hit_points: MaxHitPoints,
	pub armor_class: ArmorClass,
	pub actions: Vec<Action>,
	pub description: DerivedDescription,
	pub flags: EnumMap<Flag, bool>,
}

impl Default for Derived {
	fn default() -> Self {
		Self {
			missing_selections: Default::default(),
			ability_scores: Default::default(),
			saving_throws: Default::default(),
			skills: Default::default(),
			other_proficiencies: Default::default(),
			speeds: Default::default(),
			senses: Default::default(),
			defenses: Default::default(),
			features: Default::default(),
			max_hit_points: Default::default(),
			armor_class: Default::default(),
			actions: Default::default(),
			description: Default::default(),
			flags: enum_map! {
				Flag::ArmorStrengthRequirement => true,
			},
		}
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct SavingThrows {
	by_ability: EnumMap<Ability, ProficiencyModifiers>,
	general_modifiers: ModifierMap,
}
impl SavingThrows {
	pub fn add_proficiency(&mut self, ability: Ability, source: PathBuf) {
		self.by_ability[ability]
			.proficiency
			.push(proficiency::Level::Full, source);
	}

	pub fn add_modifier(
		&mut self,
		ability: Option<Ability>,
		modifier: Modifier,
		target: Option<String>,
		source: PathBuf,
	) {
		match ability {
			Some(ability) => &mut self.by_ability[ability].modifiers,
			None => &mut self.general_modifiers,
		}
		.insert(modifier, (target, source).into());
	}

	pub fn get_prof(&self, ability: Ability) -> &AttributedValue<proficiency::Level> {
		&self.by_ability[ability].proficiency
	}

	pub fn general_modifiers(&self) -> &ModifierMap {
		&self.general_modifiers
	}

	pub fn ability_modifiers(&self, ability: Ability) -> &ModifierMap {
		&self.by_ability[ability].modifiers
	}

	pub fn iter_modifiers(
		&self,
	) -> impl Iterator<Item = (Option<Ability>, Modifier, &ModifierMapItem)> {
		self.by_ability
			.iter()
			.map(|(ability, saving_throw)| {
				saving_throw
					.modifiers
					.iter_all()
					.map(move |(modifier, item)| (Some(ability), modifier, item))
			})
			.flatten()
			.chain(
				self.general_modifiers
					.iter_all()
					.map(|(modifier, item)| (None, modifier, item)),
			)
	}
}
impl std::ops::Index<Ability> for SavingThrows {
	type Output = ProficiencyModifiers;
	fn index(&self, index: Ability) -> &Self::Output {
		&self.by_ability[index]
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ProficiencyModifiers {
	proficiency: AttributedValue<proficiency::Level>,
	modifiers: ModifierMap,
}
impl ProficiencyModifiers {
	pub fn proficiency(&self) -> &AttributedValue<proficiency::Level> {
		&self.proficiency
	}

	pub fn modifiers(&self) -> &ModifierMap {
		&self.modifiers
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ModifierMap {
	modifiers: EnumMap<Modifier, Vec<ModifierMapItem>>,
}
impl ModifierMap {
	pub fn insert(&mut self, modifier: Modifier, item: ModifierMapItem) {
		self.modifiers[modifier].push(item);
	}

	pub fn iter(&self) -> impl Iterator<Item = (Modifier, &Vec<ModifierMapItem>)> {
		self.modifiers.iter()
	}

	pub fn get(&self, modifier: Modifier) -> &Vec<ModifierMapItem> {
		&self.modifiers[modifier]
	}

	pub fn iter_all(&self) -> impl Iterator<Item = (Modifier, &ModifierMapItem)> {
		self.modifiers
			.iter()
			.map(|(modifier, items)| items.iter().map(move |item| (modifier, item)))
			.flatten()
	}
}
#[derive(Clone, Default, PartialEq, Debug)]
pub struct ModifierMapItem {
	pub context: Option<String>,
	pub source: PathBuf,
}
impl From<(Option<String>, PathBuf)> for ModifierMapItem {
	fn from((context, source): (Option<String>, PathBuf)) -> Self {
		Self { context, source }
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Skills {
	ability_modifiers: EnumMap<Ability, ModifierMap>,
	skills: EnumMap<Skill, ProficiencyModifiers>,
}
impl Skills {
	pub fn add_proficiency(&mut self, skill: Skill, level: proficiency::Level, source: PathBuf) {
		self.skills[skill].proficiency.push(level, source);
	}

	pub fn add_ability_modifier(
		&mut self,
		ability: Ability,
		modifier: Modifier,
		context: Option<String>,
		source: PathBuf,
	) {
		self.ability_modifiers[ability].insert(modifier, (context, source).into());
	}

	pub fn add_skill_modifier(
		&mut self,
		skill: Skill,
		modifier: Modifier,
		context: Option<String>,
		source: PathBuf,
	) {
		self.skills[skill]
			.modifiers
			.insert(modifier, (context, source).into());
	}

	pub fn proficiency(&self, skill: Skill) -> &AttributedValue<proficiency::Level> {
		self.skills[skill].proficiency()
	}

	pub fn ability_modifiers(&self, ability: Ability) -> &ModifierMap {
		&self.ability_modifiers[ability]
	}

	pub fn skill_modifiers(&self, skill: Skill) -> &ModifierMap {
		&self.skills[skill].modifiers
	}

	pub fn iter_ability_modifiers(
		&self,
		ability: Ability,
	) -> impl Iterator<Item = (Modifier, &Vec<ModifierMapItem>)> {
		self.ability_modifiers[ability].iter()
	}

	pub fn iter_skill_modifiers(
		&self,
		skill: Skill,
	) -> impl Iterator<Item = (Modifier, &Vec<ModifierMapItem>)> {
		self.ability_modifiers[skill.ability()]
			.iter()
			.chain(self.skills[skill].modifiers().iter())
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Defenses(EnumMap<Defense, Vec<DefenseEntry>>);
#[derive(Clone, PartialEq, Debug)]
pub struct DefenseEntry {
	pub damage_type: Option<Value<DamageType>>,
	pub context: Option<String>,
	pub source: PathBuf,
}
impl Defenses {
	pub fn push(
		&mut self,
		kind: Defense,
		damage_type: Option<Value<DamageType>>,
		context: Option<String>,
		source: PathBuf,
	) {
		self.0[kind].push(DefenseEntry {
			damage_type,
			context,
			source,
		});
	}
}
impl std::ops::Deref for Defenses {
	type Target = EnumMap<Defense, Vec<DefenseEntry>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct DerivedDescription {
	pub life_expectancy: i32,
	pub max_height: (i32, RollSet),
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct MaxHitPoints(i32, BTreeMap<PathBuf, i32>);
impl MaxHitPoints {
	pub fn push(&mut self, bonus: i32, source: PathBuf) {
		self.0 = self.0.saturating_add(bonus);
		self.1.insert(source, bonus);
	}

	pub fn value(&self) -> u32 {
		self.0.max(0) as u32
	}

	pub fn sources(&self) -> &BTreeMap<PathBuf, i32> {
		&self.1
	}
}
