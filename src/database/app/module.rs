use crate::database::Record;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Module {
	pub id: String,
	pub system: String,
}

impl Record for Module {
	fn store_id() -> &'static str {
		"modules"
	}
}
