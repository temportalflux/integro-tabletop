use std::path::Path;

use yew::prelude::*;

/// A wrapper for a yew reducer state through
/// which the underlying data can be mutated by components.
#[derive(Clone, PartialEq)]
pub struct ContextMut<T: Reducible>(UseReducerHandle<T>);

impl<T> From<UseReducerHandle<T>> for ContextMut<T>
where
	T: Reducible,
{
	fn from(value: UseReducerHandle<T>) -> Self {
		Self(value)
	}
}

impl<T> ContextMut<T>
where
	T: Reducible,
{
	pub fn dispatch(&self, value: T::Action) {
		self.0.dispatch(value);
	}
}

impl<T> ContextMut<T>
where
	T: Reducible<Action = Callback<T, T>>,
{
	pub fn mutate<F>(&self, callback: F)
	where
		F: Fn(&mut T) + 'static,
	{
		self.dispatch(Callback::from(move |mut data| {
			callback(&mut data);
			data
		}));
	}

	pub fn new_mutator<F, I>(&self, callback: F) -> Callback<I>
	where
		T: 'static,
		F: Fn(&mut T) + 'static,
		I: 'static,
	{
		let ctx = self.0.clone();
		let mutator = std::rc::Rc::new(callback);
		Callback::from(move |_: I| {
			let mutator = mutator.clone();
			ctx.dispatch(Callback::from(move |mut data| {
				(*mutator)(&mut data);
				data
			}))
		})
	}
}

impl<T> std::ops::Deref for ContextMut<T>
where
	T: Reducible,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}

pub fn as_feature_path_text(path: &Path) -> Option<String> {
	use convert_case::{Case, Casing};
	if path.components().count() == 0 {
		return None;
	}
	Some(
		path.components()
			.map(|item| item.as_os_str().to_str().unwrap().to_case(Case::Title))
			.collect::<Vec<_>>()
			.join(" > "),
	)
}

pub fn as_feature_paths_html<'i, I>(iter: I) -> Option<String>
where
	I: Iterator<Item = &'i std::path::PathBuf>,
{
	as_feature_paths_html_custom(
		iter,
		|path| ((), path.as_path()),
		|_, path_str| format!("<div>{path_str}</div>"),
	)
}

pub fn as_feature_paths_html_custom<'i, I, T, U, FSplit, FRender>(
	iter: I,
	split_item: FSplit,
	item_as_html: FRender,
) -> Option<String>
where
	T: 'static,
	U: 'static,
	I: Iterator<Item = &'i T>,
	FSplit: Fn(&'i T) -> (U, &std::path::Path) + 'static,
	FRender: Fn(U, String) -> String + 'static,
{
	let data = iter
		.filter_map(|item| {
			let (item, path) = split_item(item);
			crate::data::as_feature_path_text(path).map(|path| (item, path))
		})
		.map(|(item, src)| item_as_html(item, src))
		.collect::<Vec<_>>();
	match data.is_empty() {
		true => None,
		false => Some(data.join("\n")),
	}
}
