use crate::{
	kdl_ext::{FromKDL, NodeContext},
	system::{
		core::SourceId,
		dnd5e::{DnD5e, SystemComponent},
	},
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Feat {}

crate::impl_kdl_node!(Feat, "feat");

impl SystemComponent for Feat {
	type System = DnD5e;

	fn add_component(mut self, source_id: SourceId, system: &mut Self::System) {}
}

impl FromKDL for Feat {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		Ok(Self {})
	}
}
