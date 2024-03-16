use super::Character;
use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::BoxedMutator,
		mutator::{self, ReferencePath},
		Block, SourceId,
	},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

/// Contains mutators and features which are applied to every character using the module it is present in.
#[derive(Clone, PartialEq, Debug)]
pub struct DefaultsBlock {
	pub source_id: Option<SourceId>,
	pub mutators: Vec<BoxedMutator>,
}

kdlize::impl_kdl_node!(DefaultsBlock, "defaults");

impl Block for DefaultsBlock {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!(null)
	}
}

impl mutator::Group for DefaultsBlock {
	type Target = Character;

	fn set_data_path(&self, parent: &ReferencePath) {
		for mutator in &self.mutators {
			mutator.set_data_path(parent);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &ReferencePath) {
		for mutator in &self.mutators {
			stats.apply(mutator, parent);
		}
	}
}

impl FromKdl<NodeContext> for DefaultsBlock {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mutators = node.query_all_t("scope() > mutator")?;
		Ok(Self {
			source_id: None,
			mutators,
		})
	}
}

impl AsKdl for DefaultsBlock {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_children_t(("mutator", self.mutators.iter()));
		node
	}
}

// TODO AsKdl: from/as tests for defaults block
