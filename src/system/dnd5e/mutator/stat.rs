use crate::{
	kdl_ext::{NodeContext, NodeReader},
	system::dnd5e::data::character::StatOperation,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub struct StatMutator {
	pub stat_name: String,
	pub operation: StatOperation,
}

impl FromKdl<NodeContext> for StatMutator {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut NodeReader) -> anyhow::Result<Self> {
		let stat_name = node.next_str_req()?.to_owned();
		let operation = StatOperation::from_kdl(node)?;
		Ok(Self { stat_name, operation })
	}
}

impl AsKdl for StatMutator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.stat_name.clone());
		node += self.operation.as_kdl();
		node
	}
}
