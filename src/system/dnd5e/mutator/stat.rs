use crate::{
	kdl_ext::{NodeContext, NodeReader},
	system::dnd5e::data::character::StatOperation,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub struct StatMutator {
	pub stat_name: Option<String>,
	pub operation: StatOperation,
}

impl FromKdl<NodeContext> for StatMutator {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut NodeReader) -> anyhow::Result<Self> {
		let stat_name = match node.next_str_req()? {
			"All" => None,
			name => Some(name.to_owned()),
		};
		let operation = StatOperation::from_kdl(node)?;
		Ok(Self { stat_name, operation })
	}
}

impl AsKdl for StatMutator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.stat_name.as_ref().map(String::as_str).unwrap_or("All"));
		node += self.operation.as_kdl();
		node
	}
}
