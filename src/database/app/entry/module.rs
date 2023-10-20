use crate::database::Entry;
use database::{IndexType, QueryExt};

pub struct Module {
	pub module: String,
}

impl IndexType for Module {
	type Record = Entry;

	fn name() -> &'static str {
		"module"
	}

	fn keys() -> &'static [&'static str] {
		&["module"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([&self.module])
	}
}
