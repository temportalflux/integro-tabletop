use crate::{
	kdl_ext::{FromKDL, NodeContext},
	system::{
		dnd5e::{SystemComponent},
	},
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Feat {}

crate::impl_kdl_node!(Feat, "feat");

impl SystemComponent for Feat {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!(null)
	}
}

impl FromKDL for Feat {
	fn from_kdl(_node: &kdl::KdlNode, _ctx: &mut NodeContext) -> anyhow::Result<Self> {
		Ok(Self {})
	}
}
