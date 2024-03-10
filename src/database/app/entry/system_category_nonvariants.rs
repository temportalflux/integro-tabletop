use crate::database::Entry;
use database::{IndexType, QueryExt};
use wasm_bindgen::JsValue;

pub struct SystemCategoryNonvariants {
	pub system: String,
	pub category: String,
}

impl IndexType for SystemCategoryNonvariants {
	type Record = Entry;

	fn name() -> &'static str {
		"system_category_nonvariants"
	}

	fn keys() -> &'static [&'static str] {
		&["system", "category", "generated"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([
			JsValue::from_str(&self.system),
			JsValue::from_str(&self.category),
			JsValue::from_bool(false),
		])
	}
}
