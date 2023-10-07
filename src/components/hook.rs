use gloo_events::{EventListener, EventListenerOptions};
use std::borrow::Cow;
use yew::prelude::*;
use yew_hooks::*;

#[hook]
pub fn use_document_event<T, F, E>(event_type: T, callback: F)
where
	T: Into<Cow<'static, str>>,
	F: Fn(E) + 'static,
	E: From<wasm_bindgen::JsValue>,
{
	let callback = use_latest(callback);
	use_effect_with_deps(
		move |event_type: &Cow<'static, str>| {
			let document = gloo_utils::document();
			let listener = EventListener::new_with_options(
				&document,
				event_type.clone(),
				EventListenerOptions::default(),
				move |event| {
					(*callback.current())(wasm_bindgen::JsValue::from(event).into());
				},
			);
			move || drop(listener)
		},
		event_type.into(),
	);
}

#[hook]
pub fn use_document_visibility<F>(callback: F)
where
	F: Fn(web_sys::VisibilityState) + 'static,
{
	let callback = use_latest(callback);
	use_document_event("visibilitychange", move |_: wasm_bindgen::JsValue| {
		let document = gloo_utils::document();
		(*callback.current())(document.visibility_state());
	});
}
