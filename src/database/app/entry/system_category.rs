use crate::database::Entry;
use database::{IndexType, QueryExt};

pub struct SystemCategory {
	pub system: String,
	pub category: String,
}

impl IndexType for SystemCategory {
	type Record = Entry;

	fn name() -> &'static str {
		"system_category"
	}

	fn keys() -> &'static [&'static str] {
		&["system", "category"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([&self.system, &self.category])
	}
}
