use crate::database::Entry;
use database::{IndexType, QueryExt};
use wasm_bindgen::JsValue;

pub type EntryVariantInSystemWithType = SystemCategoryVariants;

pub struct SystemCategoryVariants {
	pub system: String,
	pub category: String,
	pub variants_only: bool,
}

impl SystemCategoryVariants {
	pub fn new<T: crate::system::Block>(system: impl Into<String>, variants: bool) -> Self {
		Self {
			system: system.into(),
			category: T::id().into(),
			variants_only: variants,
		}
	}
}

impl IndexType for SystemCategoryVariants {
	type Record = Entry;

	fn name() -> &'static str {
		"system_category_variants"
	}

	fn keys() -> &'static [&'static str] {
		&["system", "category", "generated"]
	}

	fn as_query(&self) -> Result<idb::Query, idb::Error> {
		idb::Query::from_items([
			JsValue::from_str(&self.system),
			JsValue::from_str(&self.category),
			JsValue::from_f64(match self.variants_only {
				true => 1f64,
				false => 0f64,
			}),
		])
	}
}
