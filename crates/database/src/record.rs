// The data type for all entries in a given ObjectStore.
pub trait Record: serde::Serialize {
	fn store_id() -> &'static str;
	fn key(&self) -> Option<String> {
		None
	}
	fn as_value(&self) -> Result<wasm_bindgen::JsValue, serde_wasm_bindgen::Error> {
		Ok(self.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
	}
}
