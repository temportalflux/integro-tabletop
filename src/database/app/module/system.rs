use crate::database::Module;
use database::{IndexType, QueryExt};

#[derive(Default)]
pub struct System {
	pub system: String,
}

impl IndexType for System {
	type Record = Module;

	fn name() -> &'static str {
		"system"
	}

	fn keys() -> &'static [&'static str] {
		&["systems"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([&self.system])
	}
}
