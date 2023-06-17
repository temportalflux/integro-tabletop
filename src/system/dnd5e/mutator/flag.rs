use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{bounded::BoundValue, character::Character, description, Ability},
	utility::{InvalidEnumStr, Mutator},
};
use enum_map::Enum;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Debug, EnumSetType, Enum)]
pub enum Flag {
	// TODO: Test the usage of ArmorStrengthRequirement, w/ & w/o armor that has a req
	ArmorStrengthRequirement,
}

impl ToString for Flag {
	fn to_string(&self) -> String {
		match self {
			Self::ArmorStrengthRequirement => "ArmorStrengthRequirement",
		}
		.into()
	}
}

impl FromStr for Flag {
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"ArmorStrengthRequirement" => Ok(Self::ArmorStrengthRequirement),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}

// TODO: Test logic and from_kdl for SetFlag
#[derive(Clone, Debug, PartialEq)]
pub struct SetFlag {
	pub flag: Flag,
	pub value: bool,
}

crate::impl_trait_eq!(SetFlag);
crate::impl_kdl_node!(SetFlag, "flag");

impl Mutator for SetFlag {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: SetFlag description
		description::Section::default()
	}

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		stats.flags_mut()[self.flag] = self.value;
	}
}

impl FromKDL for SetFlag {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let flag = Flag::from_str(node.get_str_req(ctx.consume_idx())?)?;
		let value = node.get_bool_req(ctx.consume_idx())?;
		Ok(Self { flag, value })
	}
}
// TODO AsKdl: tests for SetFlag
impl AsKdl for SetFlag {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default()
			.with_entry(self.flag.to_string())
			.with_entry(self.value)
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorStrengthRequirement {
	pub score: u32,
}

crate::impl_trait_eq!(ArmorStrengthRequirement);
crate::impl_kdl_node!(ArmorStrengthRequirement, "armor_strength_requirement");

impl Mutator for ArmorStrengthRequirement {
	type Target = Character;

	fn dependencies(&self) -> crate::utility::Dependencies {
		["ability_score_finalize", "flag", "speed"].into()
	}

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: ArmorStrengthRequirement description
		description::Section::default()
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		if !stats.flags()[Flag::ArmorStrengthRequirement] {
			return;
		}
		if *stats.ability_scores().get(Ability::Strength).score() >= self.score {
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
				.insert(speed, BoundValue::Subtract(10), parent.to_owned());
		}
	}
}
impl AsKdl for ArmorStrengthRequirement {
	fn as_kdl(&self) -> NodeBuilder {
		// STUB: not available to documents
		NodeBuilder::default()
	}
}
