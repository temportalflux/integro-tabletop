use crate::database::Module;
use database::{IndexType, QueryExt};

pub type ModuleInSystem = System;

#[derive(Default)]
pub struct System {
	pub system: String,
}

impl System {
	pub fn new(system: impl Into<String>) -> Self {
		Self { system: system.into() }
	}
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
