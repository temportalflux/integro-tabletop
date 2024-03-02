use crate::{kdl_ext::NodeContext, system::core::SourceId, utility::NotInList};

use kdlize::{AsKdl, FromKdl, NodeBuilder};

pub mod item;

#[derive(Clone, PartialEq, Debug)]
pub enum Generator {
	// generates variants of items
	Item(item::Generator),
}

impl Generator {
	pub fn id(&self) -> &SourceId {
		match &self {
			Self::Item(item) => item.id(),
		}
	}
}

impl FromKdl<NodeContext> for Generator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"item" => Ok(Self::Item(item::Generator::from_kdl(node)?)),
			kind => Err(NotInList(kind.into(), vec!["item"]).into()),
		}
	}
}

impl AsKdl for Generator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Item(item) => {
				node.push_entry("item");
				node += item.as_kdl();
				node
			}
		}
	}
}
