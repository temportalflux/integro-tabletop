use std::ops::Bound;
use any_range::AnyRange;
use yew::prelude::*;
use wasm_bindgen::prelude::{Closure, JsCast};

trait BoundExt<T> {
	fn value(&self) -> Option<T>;
}
impl<T> BoundExt<T> for Bound<T> where T: Copy {
	fn value(&self) -> Option<T> {
		match self {
			Bound::Included(v) | Bound::Excluded(v) => Some(*v),
			Bound::Unbounded => None,
		}
	}
}

#[derive(Clone, PartialEq, Properties)]
pub struct ViewScalerProps {
	pub ranges: Vec<AnyRange<f64>>,
	
	/// The precision the scale operation is rounded to.
	#[prop_or(Some(1000.0))]
	pub precision: Option<f64>,

	#[prop_or_default]
	pub children: Children,
}

#[hook]
fn use_on_window_resize<F>(callback: F) where F: Fn() + 'static {
	let callback = std::rc::Rc::new(callback);
	let js_on_resize = use_memo(
		{
			move |_| {
				let callback = callback.clone();
				Closure::<dyn Fn(web_sys::UiEvent)>::new(move |_event: web_sys::UiEvent| {
					(*callback)();
				})
			}
		},
		(),
	);
	let window = web_sys::window().unwrap();
	window.set_onresize(Some((*js_on_resize).as_ref().unchecked_ref()));
}

#[function_component]
pub fn ViewScaler(ViewScalerProps { ranges, precision, children }: &ViewScalerProps) -> Html {
	fn get_width() -> f64 {
		let window = web_sys::window().unwrap();
		window.inner_width().ok().map(|js| js.as_f64()).flatten().unwrap_or(0.0)
	}

	let window_width = use_state_eq(|| get_width());
	use_on_window_resize({
		let window_width = window_width.clone();
		move || {
			window_width.set(get_width());
		}
	});

	let real_width = *window_width;
	let target_range = ranges.iter().find(|range| {
		let start = range.start_bound().value();
		let end = range.end_bound().value();
		match (start, end) {
			(None, None) => false,
			(Some(start), None) => *start < real_width,
			(None, Some(end)) => real_width < *end,
			(Some(start), Some(end)) => *start < real_width && real_width < *end,
		}
	});
	let Some(target_range) = target_range else { return html!(<div>{children.clone()}</div>); };
	let start = target_range.start_bound().value().cloned();
	let end = target_range.end_bound().value().cloned();
	let start = start.unwrap_or(real_width);

	let target = match end {
		Some(end) if end < real_width => end,
		_ => real_width,
	};
	let mut scale = target / start;
	if let Some(precision) = *precision {
		scale = (scale * precision).round() / precision;
	}
	let style = format!("transform-origin: top; transform: scale({scale});");
	html! {
		<div {style}>
			{children.clone()}
		</div>
	}
}

