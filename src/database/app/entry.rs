use crate::{database::Record, system::core::SourceId};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

mod module_system;
pub use module_system::*;
mod system;
pub use system::*;
mod system_category;
pub use system_category::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
	pub id: String,
	pub module: String,
	pub system: String,
	pub category: String,
	pub version: Option<String>,
	pub kdl: String,
}

impl Record for Entry {
	fn store_id() -> &'static str {
		"entries"
	}
}

impl Entry {
	pub fn source_id(&self) -> SourceId {
		let mut id = SourceId::from_str(&self.id).unwrap();
		id.version = self.version.clone();
		id
	}
}
