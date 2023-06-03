use crate::database::{app::Module, IndexType, QueryExt};

#[derive(Default)]
pub struct NameSystem {
	pub name: String,
	pub system: String,
}

impl IndexType for NameSystem {
	type Record = Module;

	fn name() -> &'static str {
		"name_system"
	}

	fn keys() -> &'static [&'static str] {
		&["name", "system"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([&self.name, &self.system])
	}
}
