use crate::{
	kdl_ext::{FromKDL, NodeContext},
	system::{
		core::SourceId,
		dnd5e::{DnD5e, SystemComponent},
	},
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Spell {}

crate::impl_kdl_node!(Spell, "spell");

impl SystemComponent for Spell {
	type System = DnD5e;

	fn add_component(self, _source_id: SourceId, _system: &mut Self::System) {}
}

impl FromKDL for Spell {
	fn from_kdl(_node: &kdl::KdlNode, _ctx: &mut NodeContext) -> anyhow::Result<Self> {
		Ok(Self {})
	}
}
