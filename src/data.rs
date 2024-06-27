use std::path::Path;

mod user;
pub use user::*;

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
	as_feature_paths_html_custom(iter, |path| ((), path.as_path()), |_, path_str| format!("<div>{path_str}</div>"))
}

pub fn as_feature_paths_html_custom<'i, I, T, U, FSplit, FRender>(
	iter: I, split_item: FSplit, item_as_html: FRender,
) -> Option<String>
where
	T: 'i,
	U: 'i,
	I: Iterator<Item = T>,
	FSplit: 'i + Fn(T) -> (U, &'i std::path::Path),
	FRender: 'i + Fn(U, String) -> String,
{
	let items = iter.filter_map(move |item| {
		let (item, path) = split_item(item);
		crate::data::as_feature_path_text(path).map(move |path| (item, path))
	});
	let items = items.map(move |(item, src)| item_as_html(item, src));
	let data = items.collect::<Vec<_>>();
	match data.is_empty() {
		true => None,
		false => Some(data.join("\n")),
	}
}
