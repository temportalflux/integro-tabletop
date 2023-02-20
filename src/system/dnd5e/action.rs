use uuid::Uuid;

use super::{character::State, evaluator::Evaluator, roll::Roll, Ability, BoxedFeature, Value};

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

#[derive(Clone, PartialEq)]
pub struct Attack {
	pub kind: AttackKindValue,
	pub check: AttackCheckKind,
	pub area_of_effect: Option<(AreaOfEffect, usize)>,
	pub damage_roll: (Option<Roll>, /*bonus*/ Value<i32>),
	pub damage_type: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AttackKind {
	Melee,
	Ranged,
}
#[derive(Clone, Copy, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum AreaOfEffect {
	Cone,
	Cube,
	Cylinder,
	Line,
	Sphere,
	Square,
}

#[derive(Clone, PartialEq)]
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

#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
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

	fn evaluate(&self, state: &State) -> Self::Item {
		self.value(state)
	}
}
impl AttackCheckKind {
	pub fn value(&self, state: &State) -> i32 {
		match self {
			Self::AttackRoll {
				ability,
				proficient,
			} => {
				let proficient = proficient.evaluate(state);
				state.ability_modifier(*ability, proficient.into())
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

#[derive(Clone, Copy, PartialEq)]
pub enum RangeKind {
	OnlySelf,
	Touch,
	Bounded, // abide by the short/long dist range
	Sight,
	Unlimited,
}
