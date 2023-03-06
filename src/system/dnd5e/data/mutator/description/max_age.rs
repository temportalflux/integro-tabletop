use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddLifeExpectancy(pub i32);

impl crate::utility::TraitEq for AddLifeExpectancy {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for AddLifeExpectancy {
	fn id() -> &'static str {
		"extend_life_expectancy"
	}
}

impl Mutator for AddLifeExpectancy {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		stats.derived_description_mut().life_expectancy += self.0;
	}
}

impl FromKDL<DnD5e> for AddLifeExpectancy {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		Ok(Self(node.get_i64(value_idx.next())? as i32))
	}
}

// TODO: Test AddLifeExpectancy
