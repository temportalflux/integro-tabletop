use crate::{kdl_ext::NodeContext, system::dnd5e::data::item::equipment::Equipment};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct AttunementExtension {
}

impl AttunementExtension {
	pub fn apply_to(&self, _equipment: &mut Equipment) -> anyhow::Result<()> {
		//let mut attunement = equipment.attunement.clone().unwrap_or_default();
		//equipment.attunement = Some(attunement);
		Ok(())
	}
}

impl FromKdl<NodeContext> for AttunementExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(_node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(Self { })
	}
}

impl AsKdl for AttunementExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		node
	}
}
