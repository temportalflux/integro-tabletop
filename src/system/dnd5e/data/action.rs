use crate::{
	system::dnd5e::{
		data::{character::Character, roll::Roll, Ability, BoxedFeature},
		Value,
	},
	utility::Evaluator,
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
	pub area_of_effect: Option<AreaOfEffect>,
	pub damage: Option<DamageRoll>,
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
	Cone {
		width: i32,
	},
	Cube,
	Cylinder,
	Line {
		width: i32,
		length: i32,
	},
	Sphere {
		radius: i32,
	},
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
	type Context = Character;
	type Item = i32;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
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
	pub damage_type: String,
	pub additional_bonuses: Vec<(i32, PathBuf)>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DamageType {
	Acid,
	Bludgeoning,
	Cold,
	Fire,
	Force,
	Lightning,
	Necrotic,
	Piercing,
	Poison,
	Psychic,
	Radiant,
	Slashing,
	Thunder,
}
impl DamageType {
	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Acid => "Acid",
			Self::Bludgeoning => "Bludgeoning",
			Self::Cold => "Cold",
			Self::Fire => "Fire",
			Self::Force => "Force",
			Self::Lightning => "Lightning",
			Self::Necrotic => "Necrotic",
			Self::Piercing => "Piercing",
			Self::Poison => "Poison",
			Self::Psychic => "Psychic",
			Self::Radiant => "Radiant",
			Self::Slashing => "Slashing",
			Self::Thunder => "Thunder",
		}
	}

	pub fn description(&self) -> &'static str {
		match self {
			Self::Acid => "The corrosive spray of an adult black dragon's breath and the dissolving \
			enzymes secreted by a black pudding deal acid damage.",
			Self::Bludgeoning => "Blunt force attacks--hammers, falling, constriction, \
			and the like--deal bludgeoning damage.",
			Self::Cold => "The infernal chill radiating from an ice devil's spear and the frigid blast \
			of a young white dragon's breath deal cold damage.",
			Self::Fire => "Ancient red dragons breathe fire, and many spells conjure flames to deal fire damage.",
			Self::Force => "Force is pure magical energy focused into a damaging form. \
			Most effects that deal force damage are spells, including magic missile and spiritual weapon.",
			Self::Lightning => "A lightning bolt spell and a blue dragon wyrmling's breath deal lightning damage.",
			Self::Necrotic => "Necrotic damage, dealt by certain undead and a spell such \
			as chill touch, withers matter and even the soul.",
			Self::Piercing => "Puncturing and impaling attacks, including spears and \
			monsters' bites, deal piercing damage.",
			Self::Poison => "Venomous stings and the toxic gas of an adult green dragon's breath deal poison damage.",
			Self::Psychic => "Mental abilities such as a psionic blast deal psychic damage.",
			Self::Radiant => "Radiant damage, dealt by a cleric's flame strike spell or an angel's \
			smiting weapon, sears the flesh like fire and overloads the spirit with power.",
			Self::Slashing => "Swords, axes, and monsters' claws deal slashing damage.",
			Self::Thunder => "A concussive burst of sound, such as the effect of the thunderwave spell, deals thunder damage.",
		}
	}
}
