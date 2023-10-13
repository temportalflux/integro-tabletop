use std::str::FromStr;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::Callback;

pub fn callback<T>() -> Callback<T, T> {
	Callback::from(|v| v)
}

pub trait InputExt {
	fn target_input(&self) -> Option<HtmlInputElement>;
	fn target_textarea(&self) -> Option<HtmlTextAreaElement>;
	fn target_select(&self) -> Option<HtmlSelectElement>;

	fn input_value(&self) -> Option<String> {
		if let Some(input) = self.target_input() {
			return Some(input.value());
		}
		if let Some(text_area) = self.target_textarea() {
			return Some(text_area.value());
		}
		None
	}

	fn input_value_t<T: FromStr>(&self) -> Option<T> {
		let Some(value) = self.input_value() else {
			return None;
		};
		let Ok(value) = value.parse::<T>() else {
			return None;
		};
		Some(value)
	}

	fn input_checked(&self) -> Option<bool> {
		let Some(input) = self.target_input() else {
			return None;
		};
		Some(input.checked())
	}

	fn select_value(&self) -> Option<String> {
		let Some(input) = self.target_select() else {
			return None;
		};
		Some(input.value())
	}

	fn select_value_t<T: FromStr>(&self) -> Option<T> {
		let Some(value) = self.select_value() else {
			return None;
		};
		let Ok(value) = value.parse::<T>() else {
			return None;
		};
		Some(value)
	}
}

impl InputExt for web_sys::Event {
	fn target_input(&self) -> Option<HtmlInputElement> {
		let Some(target) = self.target() else {
			return None;
		};
		target.dyn_into::<HtmlInputElement>().ok()
	}

	fn target_textarea(&self) -> Option<HtmlTextAreaElement> {
		let Some(target) = self.target() else {
			return None;
		};
		target.dyn_into::<HtmlTextAreaElement>().ok()
	}

	fn target_select(&self) -> Option<HtmlSelectElement> {
		let Some(target) = self.target() else {
			return None;
		};
		target.dyn_into::<HtmlSelectElement>().ok()
	}
}

impl InputExt for yew::prelude::NodeRef {
	fn target_input(&self) -> Option<HtmlInputElement> {
		let Some(node) = self.get() else {
			return None;
		};
		node.dyn_into::<HtmlInputElement>().ok()
	}

	fn target_textarea(&self) -> Option<HtmlTextAreaElement> {
		let Some(node) = self.get() else {
			return None;
		};
		node.dyn_into::<HtmlTextAreaElement>().ok()
	}

	fn target_select(&self) -> Option<HtmlSelectElement> {
		let Some(node) = self.get() else {
			return None;
		};
		node.dyn_into::<HtmlSelectElement>().ok()
	}
}

pub trait CallbackExt<I: 'static, O: 'static> {
	fn map<T, F>(self, map: F) -> Callback<I, T>
	where
		T: 'static,
		F: Fn(O) -> T + 'static;
}
impl<I, O> CallbackExt<I, O> for Callback<I, O>
where
	I: 'static,
	O: 'static,
{
	fn map<T, F>(self, map: F) -> Callback<I, T>
	where
		T: 'static,
		F: Fn(O) -> T + 'static,
	{
		Callback::from(move |input| map(self.emit(input)))
	}
}

pub trait CallbackOptExt<I: 'static, O: 'static> {
	fn on_some<F>(self, map: F) -> Callback<I, ()>
	where
		F: Fn(O) + 'static;
	fn map_some<F, T>(self, map: F) -> Callback<I, Option<T>>
	where
		T: 'static,
		F: Fn(O) -> Option<T> + 'static;
}
impl<I, O> CallbackOptExt<I, O> for Callback<I, Option<O>>
where
	I: 'static,
	O: 'static,
{
	fn on_some<F>(self, map: F) -> Callback<I, ()>
	where
		F: Fn(O) + 'static,
	{
		Callback::from(move |input| {
			if let Some(out) = self.emit(input) {
				map(out);
			}
		})
	}

	fn map_some<F, T>(self, map: F) -> Callback<I, Option<T>>
	where
		T: 'static,
		F: Fn(O) -> Option<T> + 'static,
	{
		Callback::from(move |input| match self.emit(input) {
			Some(out) => map(out),
			None => None,
		})
	}
}
