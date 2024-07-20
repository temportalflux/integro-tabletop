use crate::{kdl_ext::NodeContext, system::dnd5e::data::character::Character};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasSelection {
	key: PathBuf,
	value: String,
}

crate::impl_trait_eq!(HasSelection);
kdlize::impl_kdl_node!(HasSelection, "has_selection");

impl crate::system::Evaluator for HasSelection {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(format!("{} is {}", self.key.display(), self.value))
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		if let Some(selections) = character.get_selections_at(&self.key) {
			if selections.contains(&self.value) {
				return Ok(());
			}
		}
		Err(format!("{} is not selected", self.value))
	}
}

impl FromKdl<NodeContext> for HasSelection {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let key = Path::new(node.next_str_req()?).to_owned();
		let value = node.next_str_req()?.to_owned();
		Ok(Self { key, value })
	}
}

impl AsKdl for HasSelection {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.key.to_str().unwrap());
		node.entry(self.value.as_str());
		node
	}
}
