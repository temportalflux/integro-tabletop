use crate::{kdl_ext::NodeContext, system::dnd5e::data::character::Character};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasBundle {
	pub category: String,
	pub name: String,
}

crate::impl_trait_eq!(HasBundle);
kdlize::impl_kdl_node!(HasBundle, "bundle");

impl crate::system::Evaluator for HasBundle {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(format!("has {} {}", self.category, self.name))
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		for bundle in &character.persistent().bundles {
			if bundle.category == self.category && bundle.name == self.name {
				return Ok(());
			}
		}
		Err(format!("missing bundle {} {}", self.category, self.name))
	}
}

impl FromKdl<NodeContext> for HasBundle {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		let category = entry.type_req()?.to_owned();
		let name = entry.as_str_req()?.to_owned();
		Ok(Self { category, name })
	}
}

impl AsKdl for HasBundle {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry_typed(self.category.as_str(), self.name.as_str());
		node
	}
}
