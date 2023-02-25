use crate::system::dnd5e::{
	data::{character::Character, roll::Roll, Ability, BoxedFeature},
	evaluator::Evaluator,
	Value,
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, PartialEq, Default)]
pub struct Action {
	pub name: String,
	pub description: String,
	pub activation_kind: ActivationKind,
	pub attack: Option<Attack>,
	pub source: Option<ActionSource>,
}

#[derive(Clone, PartialEq)]
pub enum ActionSource {
	Item(Uuid),
	Feature(BoxedFeature),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attack {
	pub kind: AttackKindValue,
	pub check: AttackCheckKind,
	pub area_of_effect: Option<(AreaOfEffect, usize)>,
	pub damage_roll: DamageRoll,
	pub damage_type: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttackKind {
	Melee,
	Ranged,
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackKindValue {
	Melee {
		reach: i32,
	},
	Ranged {
		short_dist: u32,
		long_dist: u32,
		kind: Option<RangeKind>,
	},
}
impl AttackKindValue {
	pub fn kind(&self) -> AttackKind {
		match self {
			Self::Melee { .. } => AttackKind::Melee,
			Self::Ranged { .. } => AttackKind::Ranged,
		}
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AreaOfEffect {
	Cone,
	Cube,
	Cylinder,
	Line,
	Sphere,
	Square,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AttackCheckKind {
	AttackRoll {
		ability: Ability,
		proficient: Value<bool>,
	},
	SavingThrow {
		base: i32,
		dc_ability: Option<Ability>,
		proficient: Value<bool>,
		save_ability: Ability,
	},
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub enum ActivationKind {
	#[default]
	Action,
	Bonus,
	Reaction,
	Minute(u32),
	Hour(u32),
}

impl Evaluator for AttackCheckKind {
	type Item = i32;

	fn evaluate(&self, state: &Character) -> Self::Item {
		self.value(state)
	}
}
impl AttackCheckKind {
	pub fn value(&self, state: &Character) -> i32 {
		match self {
			Self::AttackRoll {
				ability,
				proficient,
			} => {
				let proficient = proficient.evaluate(state);
				state.ability_modifier(*ability, Some(proficient.into()))
			}
			Self::SavingThrow {
				base,
				dc_ability,
				proficient,
				save_ability: _,
			} => {
				let ability_bonus = dc_ability
					.as_ref()
					.map(|ability| state.ability_score(*ability).0.modifier())
					.unwrap_or_default();
				let prof_bonus = proficient
					.evaluate(state)
					.then(|| state.proficiency_bonus())
					.unwrap_or_default();
				*base + ability_bonus + prof_bonus
			}
		}
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RangeKind {
	OnlySelf,
	Touch,
	Bounded, // abide by the short/long dist range
	Sight,
	Unlimited,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct DamageRoll {
	pub roll: Option<Roll>,
	pub base_bonus: Value<i32>,
	pub additional_bonuses: Vec<(i32, PathBuf)>,
}
