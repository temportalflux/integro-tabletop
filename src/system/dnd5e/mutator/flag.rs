use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::data::{bounded::BoundValue, character::Character, description, Ability},
	system::Mutator,
	utility::InvalidEnumStr,
};
use enum_map::Enum;
use enumset::EnumSetType;
use kdlize::{AsKdl, FromKdl, NodeBuilder};
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
kdlize::impl_kdl_node!(SetFlag, "flag");

impl Mutator for SetFlag {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: SetFlag description
		description::Section::default()
	}

	fn apply(&self, stats: &mut Character, _parent: &ReferencePath) {
		stats.flags_mut()[self.flag] = self.value;
	}
}

impl FromKdl<NodeContext> for SetFlag {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let flag = node.next_str_req_t::<Flag>()?;
		let value = node.next_bool_req()?;
		Ok(Self { flag, value })
	}
}

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
kdlize::impl_kdl_node!(ArmorStrengthRequirement, "armor_strength_requirement");

impl Mutator for ArmorStrengthRequirement {
	type Target = Character;

	fn dependencies(&self) -> crate::utility::Dependencies {
		["ability_score_finalize", "flag", "speed"].into()
	}

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: ArmorStrengthRequirement description
		description::Section::default()
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		if !stats.flags()[Flag::ArmorStrengthRequirement] {
			return;
		}
		if *stats.ability_scores().get(Ability::Strength).score() >= self.score {
			return;
		}
		// If the rule is on and the ability score is not met,
		// then ensure that all movement speeds are decreased by 10.
		let speed_names = stats.speeds().iter().map(|(name, _)| name).cloned().collect::<Vec<_>>();
		for speed in speed_names {
			stats.speeds_mut().insert(speed, BoundValue::Subtract(10), parent);
		}
	}
}
impl AsKdl for ArmorStrengthRequirement {
	fn as_kdl(&self) -> NodeBuilder {
		// STUB: not available to documents
		NodeBuilder::default()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(SetFlag);

		#[test]
		fn armor_strength_requirement() -> anyhow::Result<()> {
			let doc = "mutator \"flag\" \"ArmorStrengthRequirement\" false";
			let data = SetFlag {
				flag: Flag::ArmorStrengthRequirement,
				value: false,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
