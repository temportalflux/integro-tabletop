use crate::{kdl_ext::NodeContext, system::dnd5e::data::item::equipment::Equipment};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct AttunementExtension {
	pub required: Option<bool>,
}

impl AttunementExtension {
	pub fn apply_to(&self, equipment: &mut Equipment) -> anyhow::Result<()> {
		let mut attunement = equipment.attunement.clone().unwrap_or_default();
		if let Some(required) = &self.required {
			attunement.required = *required;
		}
		equipment.attunement = Some(attunement);
		Ok(())
	}
}

impl FromKdl<NodeContext> for AttunementExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let required = node.get_bool_opt("required")?;
		Ok(Self { required })
	}
}

impl AsKdl for AttunementExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(required) = &self.required {
			node.entry(("required", *required));
		}
		node
	}
}
