use crate::database::Entry;
use database::{IndexType, QueryExt};
use wasm_bindgen::JsValue;

pub struct SystemVariants {
	system: String,
	generated: bool,
}

impl SystemVariants {
	pub fn new(system: impl Into<String>) -> Self {
		Self {
			system: system.into(),
			generated: true,
		}
	}
}

impl IndexType for SystemVariants {
	type Record = Entry;

	fn name() -> &'static str {
		"system_variants"
	}

	fn keys() -> &'static [&'static str] {
		&["system", "generated"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([JsValue::from_str(&self.system), JsValue::from_bool(self.generated)])
	}
}
