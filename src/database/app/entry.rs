use crate::database::Record;
use serde::{Deserialize, Serialize};

mod module_system;
pub use module_system::*;
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
