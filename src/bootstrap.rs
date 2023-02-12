use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub mod components;

#[wasm_bindgen(module = "bootstrap")]
extern "C" {
	pub type Tooltip;
	#[wasm_bindgen(constructor)]
	pub fn new(element: JsValue, config: JsValue) -> Tooltip;
}

pub fn initialize_tooltips() {
	let doc = web_sys::window().unwrap().document().unwrap();
	if let Ok(list) = doc.query_selector_all("[data-bs-toggle=\"tooltip\"]") {
		for idx in 0..list.length() {
			if let Some(node) = list.get(idx) {
				/*
				use wasm_bindgen::JsCast;
				use web_sys::HtmlElement;
				if let Some(element) = node.dyn_ref::<HtmlElement>() {
					log::debug!("- {:?}", element.outer_html());
				}
				*/
				Tooltip::new(node.into(), JsValue::from("{}".to_owned()));
			}
		}
	}
}
