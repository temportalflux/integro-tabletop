use crate::database::Entry;
use database::{IndexType, QueryExt};
use wasm_bindgen::JsValue;

pub struct SystemVariants {
	system: String,
	variants_only: bool,
}

impl SystemVariants {
	pub fn new(system: impl Into<String>, variants_only: bool) -> Self {
		Self {
			system: system.into(),
			variants_only,
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
		idb::Query::from_items([
			JsValue::from_str(&self.system),
			JsValue::from_f64(match self.variants_only {
				true => 1f64,
				false => 0f64,
			}),
		])
	}
}
