use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{bounded::BoundValue, character::Character, Ability},
			FromKDL, KDLNode,
		},
	},
	utility::Mutator,
	GeneralError,
};
use enum_map::Enum;
use std::{path::PathBuf, str::FromStr};

#[derive(Clone, Copy, PartialEq, Debug, Enum)]
pub enum Flag {
	// TODO: Test the usage of ArmorStrengthRequirement, w/ & w/o armor that has a req
	ArmorStrengthRequirement,
}

impl FromStr for Flag {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ArmorStrengthRequirement" => Ok(Self::ArmorStrengthRequirement),
			_ => Err(GeneralError(format!("Invalid flag {s:?}"))),
		}
	}
}

// TODO: Test logic and from_kdl for SetFlag
#[derive(Clone, Debug, PartialEq)]
pub struct SetFlag {
	pub flag: Flag,
	pub value: bool,
}

impl crate::utility::TraitEq for SetFlag {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for SetFlag {
	fn id() -> &'static str {
		"flag"
	}
}

impl Mutator for SetFlag {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		stats.flags_mut()[self.flag] = self.value;
	}
}

impl FromKDL for SetFlag {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let flag = Flag::from_str(node.get_str(value_idx.next())?)?;
		let value = node.get_bool(value_idx.next())?;
		Ok(Self { flag, value })
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorStrengthRequirement {
	pub score: u32,
	pub source_path: PathBuf,
}

impl crate::utility::TraitEq for ArmorStrengthRequirement {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Mutator for ArmorStrengthRequirement {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		"armor_strength_requirement"
	}

	fn dependencies(&self) -> crate::utility::Dependencies {
		["add_ability_score", "flag", "speed"].into()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		if !stats.flags()[Flag::ArmorStrengthRequirement] {
			return;
		}
		if *stats.ability_score(Ability::Strength).0 >= self.score {
			return;
		}
		// If the rule is on and the ability score is not met,
		// then ensure that all movement speeds are decreased by 10.
		let speed_names = stats
			.speeds()
			.iter()
			.map(|(name, _)| name)
			.cloned()
			.collect::<Vec<_>>();
		for speed in speed_names {
			stats
				.speeds_mut()
				.insert(speed, BoundValue::Subtract(10), self.source_path.clone());
		}
	}
}
