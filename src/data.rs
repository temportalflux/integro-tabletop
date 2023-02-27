use std::path::Path;

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
