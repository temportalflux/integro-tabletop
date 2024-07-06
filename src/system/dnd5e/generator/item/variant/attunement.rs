use crate::{kdl_ext::NodeContext, system::dnd5e::{data::item::equipment::Equipment, BoxedMutator}};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct AttunementExtension {
	pub mutators: Vec<BoxedMutator>,
}

impl AttunementExtension {
	pub fn apply_to(&self, equipment: &mut Equipment) -> anyhow::Result<()> {
		let mut attunement = equipment.attunement.clone().unwrap_or_default();
		attunement.mutators.extend(self.mutators.clone());
		equipment.attunement = Some(attunement);
		Ok(())
	}
}

impl FromKdl<NodeContext> for AttunementExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mutators = node.query_all_t("scope() > mutator")?;
		Ok(Self { mutators })
	}
}

impl AsKdl for AttunementExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.children(("mutator", self.mutators.iter()));
		node
	}
}
