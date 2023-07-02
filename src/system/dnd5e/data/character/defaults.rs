use super::Character;
use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::{BoxedMutator, SystemComponent},
	},
	utility::MutatorGroup,
};
use std::path::Path;

/// Contains mutators and features which are applied to every character using the module it is present in.
#[derive(Clone, PartialEq, Debug)]
pub struct DefaultsBlock {
	pub source_id: Option<SourceId>,
	pub mutators: Vec<BoxedMutator>,
}

crate::impl_kdl_node!(DefaultsBlock, "defaults");

impl SystemComponent for DefaultsBlock {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!(null)
	}
}

impl MutatorGroup for DefaultsBlock {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		for mutator in &self.mutators {
			mutator.set_data_path(parent);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		for mutator in &self.mutators {
			stats.apply(mutator, parent);
		}
	}
}

impl FromKDL for DefaultsBlock {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}
		Ok(Self {
			source_id: None,
			mutators,
		})
	}
}

impl AsKdl for DefaultsBlock {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
		}
		node
	}
}

// TODO AsKdl: from/as tests for defaults block
