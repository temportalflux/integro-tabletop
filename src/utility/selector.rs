use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	GeneralError,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Selector<T> {
	Specific(T),
	AnyOf { id: Option<String>, options: Vec<T> },
	Any { id: Option<String> },
}

impl<T> Selector<T> {
	pub fn id(&self) -> Option<&str> {
		match self {
			Self::Specific(_) => None,
			Self::AnyOf { id, options: _ } => id.as_ref(),
			Self::Any { id } => id.as_ref(),
		}
		.map(String::as_str)
	}

	pub fn from_kdl(
		node: &kdl::KdlNode,
		entry: &kdl::KdlEntry,
		value_idx: &mut ValueIdx,
		map_value: impl Fn(&kdl::KdlValue) -> anyhow::Result<T>,
	) -> anyhow::Result<Self> {
		let key = entry.value().as_string().ok_or(GeneralError(format!(
			"Selector key must be a string, but {entry:?} of {node:?} is not."
		)))?;
		match key {
			"Specific" => {
				let idx = value_idx.next();
				let value = node.get(idx).ok_or(GeneralError(format!(
					"Missing specific selector value at index {idx} of {node:?}"
				)))?;
				Ok(Self::Specific(map_value(value)?))
			}
			"AnyOf" => {
				let id = node.get_str_opt("id")?.map(str::to_owned);
				let mut options = Vec::new();
				for kdl_value in node.query_get_all("option", 0)? {
					options.push(map_value(kdl_value)?);
				}
				Ok(Self::AnyOf { id, options })
			}
			"Any" => {
				let id = node.get_str_opt("id")?.map(str::to_owned);
				Ok(Self::Any { id })
			}
			_ => Err(GeneralError(format!(
				"Invalid selector key {key:?}, expected Specific, Any, or AnyOf."
			))
			.into()),
		}
	}
}
