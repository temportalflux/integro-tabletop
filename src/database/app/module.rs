use crate::{database::Record, system::core::ModuleId};
use serde::{Deserialize, Serialize};

mod system;
pub use system::*;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Module {
	pub module_id: ModuleId,
	pub name: String,
	pub system: String,
	pub version: String,
}

impl Record for Module {
	fn store_id() -> &'static str {
		"modules"
	}

	fn key(&self) -> Option<String> {
		None // Some(self.module_id.to_string())
	}
}
