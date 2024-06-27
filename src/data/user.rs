use crate::{
	auth::OAuthProvider,
	kdl_ext::NodeContext,
	system::{ModuleId, SourceId},
};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder, NodeReader,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserId {
	pub provider: OAuthProvider,
	pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct UserSettings {
	pub friends: Vec<UserId>,
}

kdlize::impl_kdl_node!(UserSettings, "user_settings");

impl UserSettings {
	pub fn homebrew_id(module: &ModuleId) -> SourceId {
		SourceId {
			module: Some(module.clone()),
			system: None,
			path: Path::new("user_settings.kdl").into(),
			version: None,
			variant: None,
		}
	}
}

impl FromKdl<NodeContext> for UserSettings {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut NodeReader<NodeContext>) -> Result<Self, Self::Error> {
		let mut friends = Vec::new();
		for mut node in node.query_all("scope() > friend")? {
			let entry = node.next_req()?;
			let provider = OAuthProvider::from_str(entry.type_req()?)?;
			let id = entry.as_str_req()?.to_owned();
			friends.push(UserId { provider, id });
		}

		Ok(Self { friends })
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
