use crate::{database::Record, system::core::ModuleId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod system;
pub use system::*;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Module {
	pub module_id: ModuleId,
	pub name: String,
	pub systems: BTreeSet<String>,
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
