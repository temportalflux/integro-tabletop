use crate::{
	kdl_ext::ValueIdx,
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{action::Action, character::Character},
			FromKDL,
		},
	},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddAction(pub Action);

crate::impl_trait_eq!(AddAction);
crate::impl_kdl_node!(AddAction, "add_action");

impl Mutator for AddAction {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some(self.0.name.clone())
	}

	fn description(&self) -> Option<String> {
		self.0.description.clone()
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.0.set_data_path(parent);
	}

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		stats.actions_mut().push(self.0.clone());
	}
}

impl FromKDL for AddAction {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		Ok(Self(Action::from_kdl(node, value_idx, node_reg)?))
	}
}
