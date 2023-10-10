use yew::prelude::*;

#[hook]
pub fn use_media_query(query: impl AsRef<str>) -> UseStateHandle<bool> {
	use wasm_bindgen::prelude::{Closure, JsCast};
	use web_sys::MediaQueryListEvent;

	let query_result = use_state_eq(|| false);
	let js_on_media_query_changed = use_memo((), {
		let query_result = query_result.clone();
		move |_| {
			Closure::<dyn Fn(MediaQueryListEvent)>::new(move |event: MediaQueryListEvent| {
				query_result.set(event.matches());
			})
		}
	});

	let window = web_sys::window().unwrap();
	let Ok(Some(media_query)) = window.match_media(query.as_ref()) else {
		return query_result;
	};

	query_result.set(media_query.matches());
	media_query.set_onchange(Some((*js_on_media_query_changed).as_ref().unchecked_ref()));

	query_result
}
