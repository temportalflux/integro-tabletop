use crate::system::core::ModuleId;
use database::Record;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod system;
pub use system::*;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Module {
	pub id: ModuleId,
	pub name: String,
	pub systems: BTreeSet<String>,
	pub version: String,
	pub remote_version: String,
	pub installed: bool,
}

impl Record for Module {
	fn store_id() -> &'static str {
		"modules"
	}

	fn key(&self) -> Option<String> {
		None // Some(self.module_id.to_string())
	}
}
