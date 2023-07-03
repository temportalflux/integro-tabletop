use crate::{database::Record, system::core::SourceId};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

mod module_system;
pub use module_system::*;
mod system;
pub use system::*;
mod system_category;
pub use system_category::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Entry {
	pub id: String,
	pub module: String,
	pub system: String,
	pub category: String,
	pub version: Option<String>,
	pub metadata: serde_json::Value,
	pub kdl: String,
	pub file_id: Option<String>,
}

impl Record for Entry {
	fn store_id() -> &'static str {
		"entries"
	}
}

impl Entry {
	pub fn source_id(&self, with_version: bool) -> SourceId {
		let mut id = SourceId::from_str(&self.id).unwrap();
		if with_version {
			id.version = self.version.clone();
		}
		id
	}

	pub fn get_meta_str(&self, key: impl AsRef<str>) -> Option<&str> {
		let Some(value) = self.metadata.get(key.as_ref()) else { return None; };
		value.as_str()
	}

	pub fn name(&self) -> Option<&str> {
		self.get_meta_str("name")
	}

	pub fn parse_kdl<T: crate::kdl_ext::FromKDL>(
		&self,
		node_reg: Arc<crate::system::core::NodeRegistry>,
	) -> Option<T> {
		// Parse the entry's kdl string:
		// kdl string to document
		let Ok(document) = self.kdl.parse::<kdl::KdlDocument>() else { return None; };
		// document to first (and hopefully only) node
		let Some(node) = document.nodes().get(0) else { return None; };
		let ctx = crate::kdl_ext::NodeContext::new(Arc::new(self.source_id(true)), node_reg);
		let Ok(value) = T::from_kdl_reader(&mut crate::kdl_ext::NodeReader::new(node, ctx)) else { return None; };
		Some(value)
	}
}
