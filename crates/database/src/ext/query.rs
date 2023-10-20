use wasm_bindgen::JsValue;

pub trait QueryExt {
	fn from_items<T: Into<JsValue>, const N: usize>(items: [T; N]) -> Result<idb::Query, idb::Error>;
}

impl QueryExt for idb::Query {
	fn from_items<T: Into<JsValue>, const N: usize>(items: [T; N]) -> Result<idb::Query, idb::Error> {
		if items.len() == 1 {
			let t_val = items.into_iter().next().unwrap();
			Ok(idb::Query::Key(t_val.into()))
		} else {
			let values = js_sys::Array::new_with_length(items.len() as u32);
			for (idx, t_val) in items.into_iter().enumerate() {
				values.set(idx as u32, t_val.into());
			}
			Ok(idb::Query::KeyRange(idb::KeyRange::only(&values)?))
		}
	}
}
