use crate::{
	kdl_ext::{NodeContext, NodeReader},
	path_map::PathMap,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::path::Path;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Notes(PathMap<String>);

impl std::ops::Deref for Notes {
	type Target = PathMap<String>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl std::ops::DerefMut for Notes {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl FromKdl<NodeContext> for Notes {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut NodeReader) -> anyhow::Result<Self> {
		let mut notes = PathMap::default();
		for mut node in node.query_all("scope() > note")? {
			let path = Path::new(node.next_str_req()?);
			let content = node.next_str_req()?.to_owned();
			notes.insert(path, content);
		}
		Ok(Self(notes))
	}
}

impl AsKdl for Notes {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for (path, content) in self.0.as_vec() {
			let Some(path_str) = path.to_str() else { continue };
			node.child(("note", {
				let mut node = NodeBuilder::default();
				node.entry(path_str);
				node.entry(content.as_str());
				node
			}));
		}
		node
	}
}
