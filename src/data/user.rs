use crate::{auth::OAuthProvider, kdl_ext::NodeContext, system::SourceId};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder, NodeReader,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct UserId {
	provider: OAuthProvider,
	id: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UserSettings {
	pub id: SourceId,
	pub friends: Vec<UserId>,
}

impl FromKdl<NodeContext> for UserSettings {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut NodeReader<NodeContext>) -> Result<Self, Self::Error> {
		let id = node.context().id().clone();

		let mut friends = Vec::new();
		for mut node in node.query_all("scope() > friend")? {
			let entry = node.next_req()?;
			let provider = OAuthProvider::from_str(entry.type_req()?)?;
			let id = entry.as_str_req()?.to_owned();
			friends.push(UserId { provider, id });
		}

		Ok(Self { id, friends })
	}
}

impl AsKdl for UserSettings {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		for user_id in &self.friends {
			node.child(("friend", {
				let mut node = NodeBuilder::default();
				node.entry_typed(user_id.provider.to_string(), user_id.id.as_str());
				node
			}));
		}

		node
	}
}
