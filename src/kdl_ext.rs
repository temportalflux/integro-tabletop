use crate::GeneralError;

pub trait DocumentQueryExt {
	fn query_bool(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<bool>;

	fn query_i64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<i64>;

	fn query_f64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<f64>;

	fn query_str(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<&str>;
}

fn query_type<'doc, T>(
	doc: &'doc kdl::KdlDocument,
	query: impl kdl::IntoKdlQuery + Clone,
	key: impl Into<kdl::NodeKey> + Clone,
	map: impl FnOnce(&'doc kdl::KdlValue) -> Result<T, &'static str>,
) -> anyhow::Result<T> {
	let query_str = match query.clone().into_query() {
		Ok(query) => format!("{query:?}"),
		_ => "[unknown]".into(),
	};
	let key_str = format!("{:?}", key.clone().into());
	let value = doc.query_get(query, key)?;
	let value = value.ok_or_else(|| {
		GeneralError(format!(
			"Node {query_str:?} is missing value at {key_str:?}"
		))
	})?;
	let value = map(value).map_err(|type_name| {
		GeneralError(format!(
			"Value at {key_str:?} of node {query_str:?} is not a {type_name}"
		))
	})?;
	Ok(value)
}

impl DocumentQueryExt for kdl::KdlDocument {
	fn query_bool(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<bool> {
		query_type(self, query, key, |value| match value.as_bool() {
			Some(value) => Ok(value),
			None => Err("bool"),
		})
	}

	fn query_i64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<i64> {
		query_type(self, query, key, |value| match value.as_i64() {
			Some(value) => Ok(value),
			None => Err("integer"),
		})
	}

	fn query_f64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<f64> {
		query_type(self, query, key, |value| match value.as_f64() {
			Some(value) => Ok(value),
			None => Err("float"),
		})
	}

	fn query_str(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<&str> {
		query_type(self, query, key, |value| match value.as_string() {
			Some(value) => Ok(value),
			None => Err("string"),
		})
	}
}

#[cfg(test)]
mod test {
	use super::DocumentQueryExt;

	fn doc_str() -> &'static str {
		r#"
			nodebool true
			nodeint 42
			nodefloat prop=10.5
			a {
				nodestr "some string"
			}
		"#
	}

	fn document() -> anyhow::Result<kdl::KdlDocument> {
		Ok(doc_str().parse::<kdl::KdlDocument>()?)
	}

	#[test]
	fn query_bool() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_bool("nodebool", 0)?, true);
		Ok(())
	}

	#[test]
	fn query_i64() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_i64("nodeint", 0)?, 42i64);
		Ok(())
	}

	#[test]
	fn query_float() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_f64("nodefloat", "prop")?, 10.5f64);
		Ok(())
	}

	#[test]
	fn query_str() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_str("a > nodestr", 0)?, "some string");
		Ok(())
	}
}
