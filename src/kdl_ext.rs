use crate::GeneralError;

pub trait DocumentQueryExt {
	fn as_document(&self) -> anyhow::Result<&kdl::KdlDocument>;

	fn query_bool(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<bool> {
		query_type(self.as_document()?, query, key, as_bool)
	}

	fn query_i64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<i64> {
		query_type(self.as_document()?, query, key, as_i64)
	}

	fn query_f64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<f64> {
		query_type(self.as_document()?, query, key, as_f64)
	}

	fn query_str(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<&str> {
		query_type(self.as_document()?, query, key, as_str)
	}

	fn query_bool_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<bool>> {
		query_type_opt(self.as_document()?, query, key, as_bool)
	}

	fn query_i64_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<i64>> {
		query_type_opt(self.as_document()?, query, key, as_i64)
	}

	fn query_f64_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<f64>> {
		query_type_opt(self.as_document()?, query, key, as_f64)
	}

	fn query_str_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<&str>> {
		query_type_opt(self.as_document()?, query, key, as_str)
	}

	fn query_bool_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<bool>> + 'doc>> {
		Ok(Box::new(query_type_all(
			self.as_document()?,
			query,
			key,
			as_bool,
		)?))
	}

	fn query_i64_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<i64>> + 'doc>> {
		Ok(Box::new(query_type_all(
			self.as_document()?,
			query,
			key,
			as_i64,
		)?))
	}

	fn query_f64_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<f64>> + 'doc>> {
		Ok(Box::new(query_type_all(
			self.as_document()?,
			query,
			key,
			as_f64,
		)?))
	}

	fn query_str_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<&str>> + 'doc>> {
		Ok(Box::new(query_type_all(
			self.as_document()?,
			query,
			key,
			as_str,
		)?))
	}
}
pub trait NodeQueryExt {
	fn as_node(&self) -> &kdl::KdlNode;

	fn entry_req(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<&kdl::KdlEntry> {
		self.as_node().entry(key.clone()).ok_or(
			GeneralError(format!(
				"Missing value at {:?} in node {:?}",
				key.into(),
				self.as_node()
			))
			.into(),
		)
	}

	fn get_bool(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<bool> {
		get_type(self.as_node(), key, as_bool)
	}

	fn get_i64(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<i64> {
		get_type(self.as_node(), key, as_i64)
	}

	fn get_f64(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<f64> {
		get_type(self.as_node(), key, as_f64)
	}

	fn get_str(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<&str> {
		get_type(self.as_node(), key, as_str)
	}

	fn get_bool_opt(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<Option<bool>> {
		get_type_opt(self.as_node(), key, as_bool)
	}

	fn get_i64_opt(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<Option<i64>> {
		get_type_opt(self.as_node(), key, as_i64)
	}

	fn get_f64_opt(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<Option<f64>> {
		get_type_opt(self.as_node(), key, as_f64)
	}

	fn get_str_opt(&self, key: impl Into<kdl::NodeKey> + Clone) -> anyhow::Result<Option<&str>> {
		get_type_opt(self.as_node(), key, as_str)
	}
}

fn query_type<'doc, T>(
	doc: &'doc kdl::KdlDocument,
	query: impl kdl::IntoKdlQuery + Clone,
	key: impl Into<kdl::NodeKey> + Clone,
	map: impl FnOnce(&'doc kdl::KdlValue) -> Result<T, &'static str>,
) -> anyhow::Result<T> {
	let key_str = format!("{:?}", key.clone().into());
	match query_type_opt(doc, query.clone(), key, map)? {
		Some(value) => Ok(value),
		None => Err(GeneralError(format!(
			"Node {:?} is missing value at {key_str:?}",
			doc.query(query).unwrap().unwrap()
		))
		.into()),
	}
}

fn query_type_opt<'doc, T>(
	doc: &'doc kdl::KdlDocument,
	query: impl kdl::IntoKdlQuery + Clone,
	key: impl Into<kdl::NodeKey> + Clone,
	map: impl FnOnce(&'doc kdl::KdlValue) -> Result<T, &'static str>,
) -> anyhow::Result<Option<T>> {
	let key_str = format!("{:?}", key.clone().into());
	let Ok(Some(value)) = doc.query_get(query.clone(), key) else { return Ok(None); };
	let value = map(value).map_err(|type_name| {
		GeneralError(format!(
			"Value at {key_str:?} of node {:?} is not a {type_name}",
			doc.query(query).unwrap().unwrap()
		))
	})?;
	Ok(Some(value))
}

fn query_type_all<'doc, T>(
	doc: &'doc kdl::KdlDocument,
	query: impl kdl::IntoKdlQuery + Clone + 'doc,
	key: impl Into<kdl::NodeKey> + Clone + 'doc,
	map: impl Fn(&'doc kdl::KdlValue) -> Result<T, &'static str> + 'doc,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<T>> + 'doc> {
	let query_str = match query.clone().into_query() {
		Ok(query) => format!("{query:?}"),
		_ => "[unknown]".into(),
	};
	let key_str = format!("{:?}", key.clone().into());
	Ok(doc.query_get_all(query, key)?.map(move |kdl_value| {
		let query_str = query_str.clone();
		let key_str = key_str.clone();
		map(kdl_value).map_err(move |type_name| {
			GeneralError(format!(
				"Value at {key_str:?} of a node in {query_str:?} is not a {type_name}"
			))
			.into()
		})
	}))
}

fn get_type<'doc, T>(
	node: &'doc kdl::KdlNode,
	key: impl Into<kdl::NodeKey> + Clone,
	map: impl FnOnce(&'doc kdl::KdlValue) -> Result<T, &'static str>,
) -> anyhow::Result<T> {
	let key_str = format!("{:?}", key.clone().into());
	match get_type_opt::<T>(node, key, map)? {
		Some(value) => Ok(value),
		None => Err(GeneralError(format!("Node {node:?} is missing value at {key_str:?}")).into()),
	}
}

fn get_type_opt<'doc, T>(
	node: &'doc kdl::KdlNode,
	key: impl Into<kdl::NodeKey> + Clone,
	map: impl FnOnce(&'doc kdl::KdlValue) -> Result<T, &'static str>,
) -> anyhow::Result<Option<T>> {
	let key_str = format!("{:?}", key.clone().into());
	let Some(value) = node.get(key) else { return Ok(None); };
	let value = map(value).map_err(|type_name| {
		GeneralError(format!(
			"Value at {key_str:?} of node {:?} is not a {type_name}",
			node.name().value()
		))
	})?;
	Ok(Some(value))
}

fn as_bool(value: &kdl::KdlValue) -> Result<bool, &'static str> {
	value.as_bool().ok_or("bool")
}
fn as_i64(value: &kdl::KdlValue) -> Result<i64, &'static str> {
	value.as_i64().ok_or("integer")
}
fn as_f64(value: &kdl::KdlValue) -> Result<f64, &'static str> {
	value.as_f64().ok_or("decimal")
}
fn as_str(value: &kdl::KdlValue) -> Result<&str, &'static str> {
	value.as_string().ok_or("string")
}

impl DocumentQueryExt for kdl::KdlDocument {
	fn as_document(&self) -> anyhow::Result<&kdl::KdlDocument> {
		Ok(&self)
	}
}
impl NodeQueryExt for kdl::KdlNode {
	fn as_node(&self) -> &kdl::KdlNode {
		self
	}
}
impl DocumentQueryExt for kdl::KdlNode {
	fn as_document(&self) -> anyhow::Result<&kdl::KdlDocument> {
		self.children()
			.ok_or(GeneralError(format!("Node {:?} has no children", self.name().value())).into())
	}
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ValueIdx(usize);
impl std::ops::Deref for ValueIdx {
	type Target = usize;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl std::ops::DerefMut for ValueIdx {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl ValueIdx {
	pub fn next(&mut self) -> usize {
		let consumed = self.0;
		self.0 += 1;
		consumed
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
