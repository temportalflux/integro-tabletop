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

	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!(null)
	}

	fn add_component(self, _source_id: SourceId, _system: &mut Self::System) {}
}

impl FromKDL for Feat {
	fn from_kdl(_node: &kdl::KdlNode, _ctx: &mut NodeContext) -> anyhow::Result<Self> {
		Ok(Self {})
	}
}
