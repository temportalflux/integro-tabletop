use super::AttributedValue;
use crate::{
	path_map::PathMap,
	system::dnd5e::{
		data::{
			action::{Action, ActivationKind, Attack, AttackCheckKind, AttackKindValue},
			mutator::{Defense, Flag},
			proficiency,
			roll::{Modifier, RollSet},
			Ability, ArmorClass, BoxedFeature, DamageRoll, DamageType, OtherProficiencies, Skill,
		},
		Value,
	},
};
use enum_map::{enum_map, EnumMap};
use std::{collections::BTreeMap, path::PathBuf};

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
			actions: vec![Action {
				name: "Unarmed Strike".into(),
				description: "Instead of using a weapon to make a melee weapon attack, \
				you can use an unarmed strike: a punch, kick, head-butt, or similar \
				forceful blow (none of which count as weapons). On a hit, an unarmed \
				strike deals bludgeoning damage equal to 1 + your Strength modifier. \
				You are proficient with your unarmed strikes."
					.into(),
				activation_kind: ActivationKind::Action,
				attack: Some(Attack {
					kind: AttackKindValue::Melee { reach: 5 },
					check: AttackCheckKind::AttackRoll {
						ability: Ability::Strength,
						proficient: Value::Fixed(true),
					},
					area_of_effect: None,
					damage: Some(DamageRoll {
						base_bonus: 1,
						damage_type: DamageType::Bludgeoning,
						..Default::default()
					}),
				}),
				..Default::default()
			}],
			description: Default::default(),
			flags: enum_map! {
				Flag::ArmorStrengthRequirement => true,
			},
		}
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AbilityScores(EnumMap<Ability, AttributedValue<i32>>);
impl AbilityScores {
	pub fn push_bonus(&mut self, ability: Ability, bonus: i32, source: PathBuf) {
		self.0[ability].push(bonus, source);
	}

	pub fn get(&self, ability: Ability) -> &AttributedValue<i32> {
		&self.0[ability]
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct SavingThrows {
	by_ability: EnumMap<
		Ability,
		(
			/*is proficient*/ AttributedValue<proficiency::Level>,
			/*adv modifiers*/ Vec<(Option<String>, PathBuf)>,
		),
	>,
	general_modifiers: Vec<(Option<String>, PathBuf)>,
}
impl SavingThrows {
	pub fn add_proficiency(&mut self, ability: Ability, source: PathBuf) {
		self.by_ability[ability]
			.0
			.push(proficiency::Level::Full, source);
	}

	pub fn add_modifier(
		&mut self,
		ability: Option<Ability>,
		target: Option<String>,
		source: PathBuf,
	) {
		match ability {
			Some(ability) => &mut self.by_ability[ability].1,
			None => &mut self.general_modifiers,
		}
		.push((target, source));
	}

	pub fn get_prof(&self, ability: Ability) -> &AttributedValue<proficiency::Level> {
		&self.by_ability[ability].0
	}

	pub fn iter_modifiers(
		&self,
	) -> impl Iterator<Item = (Option<Ability>, &Option<String>, &PathBuf)> {
		self.by_ability
			.iter()
			.map(|(ability, (_, modifiers))| {
				modifiers
					.iter()
					.map(move |(target, path)| (Some(ability), target, path))
			})
			.flatten()
			.chain(
				self.general_modifiers
					.iter()
					.map(|(target, path)| (None, target, path)),
			)
	}
}
impl std::ops::Index<Ability> for SavingThrows {
	type Output = (
		AttributedValue<proficiency::Level>,
		Vec<(Option<String>, PathBuf)>,
	);
	fn index(&self, index: Ability) -> &Self::Output {
		&self.by_ability[index]
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
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
		self.0 as u32
	}

	pub fn sources(&self) -> &BTreeMap<PathBuf, i32> {
		&self.1
	}
}
