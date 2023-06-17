use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{character::Character, description},
	utility::Mutator,
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddLifeExpectancy(pub i32);

crate::impl_trait_eq!(AddLifeExpectancy);
crate::impl_kdl_node!(AddLifeExpectancy, "extend_life_expectancy");

impl Mutator for AddLifeExpectancy {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			content: format!("Your life expectancy increases by {} years.", self.0).into(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		stats.derived_description_mut().life_expectancy += self.0;
	}
}

impl FromKDL for AddLifeExpectancy {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(node.get_i64_req(ctx.consume_idx())? as i32))
	}
}
// TODO AsKdl: from/as tests for AddLifeExpectancy
impl AsKdl for AddLifeExpectancy {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default().with_entry(self.0 as i64)
	}
}
