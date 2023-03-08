use super::roll::Roll;
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::dnd5e::{DnD5e, FromKDL},
	GeneralError,
};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct DamageRoll {
	// TODO: Implement damage which scales according to some scalar (usually class, character, or spell level)
	pub roll: Option<Roll>,
	pub base_bonus: i32,
	pub damage_type: DamageType,
	// generated (see BonusDamage mutator)
	pub additional_bonuses: Vec<(i32, PathBuf)>,
}

impl FromKDL<DnD5e> for DamageRoll {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let roll = match node.query_str_opt("roll", 0)? {
			None => None,
			Some(str) => Some(Roll::from_str(str)?),
		};
		let base_bonus = node.get_i64_opt("base")?.unwrap_or(0) as i32;
		let damage_type = DamageType::from_str(node.query_str("damage_type", 0)?)?;
		Ok(Self {
			roll,
			base_bonus,
			damage_type,
			additional_bonuses: Vec::new(),
		})
	}
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DamageType {
	Acid,
	Bludgeoning,
	Cold,
	#[default]
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

impl FromStr for DamageType {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Acid" => Ok(Self::Acid),
			"Bludgeoning" => Ok(Self::Bludgeoning),
			"Cold" => Ok(Self::Cold),
			"Fire" => Ok(Self::Fire),
			"Force" => Ok(Self::Force),
			"Lightning" => Ok(Self::Lightning),
			"Necrotic" => Ok(Self::Necrotic),
			"Piercing" => Ok(Self::Piercing),
			"Poison" => Ok(Self::Poison),
			"Psychic" => Ok(Self::Psychic),
			"Radiant" => Ok(Self::Radiant),
			"Slashing" => Ok(Self::Slashing),
			"Thunder" => Ok(Self::Thunder),
			_ => Err(GeneralError(format!("Invalid damage type {s:?}")).into()),
		}
	}
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
