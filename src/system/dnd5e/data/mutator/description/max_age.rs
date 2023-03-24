use crate::{
	kdl_ext::{NodeExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::character::Character, FromKDL},
	},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddLifeExpectancy(pub i32);

crate::impl_trait_eq!(AddLifeExpectancy);
crate::impl_kdl_node!(AddLifeExpectancy, "extend_life_expectancy");

impl Mutator for AddLifeExpectancy {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some("Age".into())
	}

	fn description(&self) -> Option<String> {
		Some(format!(
			"Your life expectancy increases by {} years.",
			self.0
		))
	}

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		stats.derived_description_mut().life_expectancy += self.0;
	}
}

impl FromKDL for AddLifeExpectancy {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		Ok(Self(node.get_i64_req(value_idx.next())? as i32))
	}
}

// TODO: Test AddLifeExpectancy
