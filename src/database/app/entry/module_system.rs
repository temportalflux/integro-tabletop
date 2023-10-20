use crate::database::Entry;
use database::{IndexType, QueryExt};

pub struct ModuleSystem {
	pub module: String,
	pub system: String,
}

impl IndexType for ModuleSystem {
	type Record = Entry;

	fn name() -> &'static str {
		"module_system"
	}

	fn keys() -> &'static [&'static str] {
		&["module", "system"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([&self.module, &self.system])
	}
}
