use crate::GeneralError;

#[derive(thiserror::Error, Debug)]
#[error("Entry \"{0}\" is missing a type identifier.")]
struct EntryMissingType(kdl::KdlEntry);

/// The kdl value did not match the expected type.
#[derive(thiserror::Error, Debug)]
#[error("Invalid value {0:?}, was expecting a {1}.")]
struct InvalidValueType(kdl::KdlValue, &'static str);

/// The node is missing an entry that was required.
#[derive(thiserror::Error, Debug)]
struct EntryMissing(kdl::KdlNode, kdl::NodeKey);
impl std::fmt::Display for EntryMissing {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.1 {
			kdl::NodeKey::Index(v) => write!(f, "Node {} is missing an entry at index {v}", self.0),
			kdl::NodeKey::Key(v) => {
				write!(
					f,
					"Node {} is missing an entry at property {}",
					self.0,
					v.value()
				)
			}
		}
	}
}

#[derive(thiserror::Error, Debug)]
enum RequiredValue {
	#[error(transparent)]
	InvalidValueType(#[from] InvalidValueType),
	#[error(transparent)]
	EntryMissing(#[from] EntryMissing),
}

trait EntryExt {
	/// Returns the type of the entry.
	/// If the entry does not have a type, None is returned.
	fn type_opt(&self) -> Option<&str>;
	/// Returns the type of the entry.
	/// If the entry does not have a type, an error is returned.
	fn type_req(&self) -> Result<&str, EntryMissingType>;

	/// Returns the value of the entry.
	/// If the value is not a bool, an error is returned.
	fn as_bool(&self) -> Result<bool, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a i64, an error is returned.
	fn as_i64(&self) -> Result<i64, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a f64, an error is returned.
	fn as_f64(&self) -> Result<f64, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a string, an error is returned.
	fn as_str(&self) -> Result<&str, InvalidValueType>;
}
impl EntryExt for kdl::KdlEntry {
	fn type_opt(&self) -> Option<&str> {
		self.ty().map(|id| id.value())
	}

	fn type_req(&self) -> Result<&str, EntryMissingType> {
		Ok(self.type_opt().ok_or(EntryMissingType(self.clone()))?)
	}

	fn as_bool(&self) -> Result<bool, InvalidValueType> {
		Ok(self
			.value()
			.as_bool()
			.ok_or(InvalidValueType(self.value().clone(), "bool"))?)
	}

	fn as_i64(&self) -> Result<i64, InvalidValueType> {
		Ok(self
			.value()
			.as_i64()
			.ok_or(InvalidValueType(self.value().clone(), "i64"))?)
	}

	fn as_f64(&self) -> Result<f64, InvalidValueType> {
		Ok(self
			.value()
			.as_f64()
			.ok_or(InvalidValueType(self.value().clone(), "f64"))?)
	}

	fn as_str(&self) -> Result<&str, InvalidValueType> {
		Ok(self
			.value()
			.as_string()
			.ok_or(InvalidValueType(self.value().clone(), "string"))?)
	}
}

trait NodeExt {
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	fn entry_opt(&self, key: impl Into<kdl::NodeKey>) -> Option<&kdl::KdlEntry>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	fn entry_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&kdl::KdlEntry, EntryMissing>;

	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a bool, an error is returned.
	fn get_bool_opt_2(
		&self,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, InvalidValueType>;
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a i64, an error is returned.
	fn get_i64_opt_2(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, InvalidValueType>;
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a f64, an error is returned.
	fn get_f64_opt_2(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, InvalidValueType>;
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a string, an error is returned.
	fn get_str_opt_2(&self, key: impl Into<kdl::NodeKey>)
		-> Result<Option<&str>, InvalidValueType>;

	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a bool, an error is returned.
	fn get_bool_req(&self, key: impl Into<kdl::NodeKey>) -> Result<bool, RequiredValue>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a i64, an error is returned.
	fn get_i64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<i64, RequiredValue>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a f64, an error is returned.
	fn get_f64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<f64, RequiredValue>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a string, an error is returned.
	fn get_str_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&str, RequiredValue>;
}

impl NodeExt for kdl::KdlNode {
	fn entry_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&kdl::KdlEntry, EntryMissing> {
		let key = key.into();
		self.entry_opt(key.clone())
			.ok_or(EntryMissing(self.clone(), key))
	}

	fn entry_opt(&self, key: impl Into<kdl::NodeKey>) -> Option<&kdl::KdlEntry> {
		self.entry(key)
	}

	fn get_bool_opt_2(
		&self,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_bool()?))
	}

	fn get_i64_opt_2(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_i64()?))
	}

	fn get_f64_opt_2(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_f64()?))
	}

	fn get_str_opt_2(
		&self,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<&str>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_str()?))
	}

	fn get_bool_req(&self, key: impl Into<kdl::NodeKey>) -> Result<bool, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_bool_opt_2(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}

	fn get_i64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<i64, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_i64_opt_2(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}

	fn get_f64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<f64, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_f64_opt_2(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}

	fn get_str_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&str, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_str_opt_2(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}
}

trait DocumentExt {
	/// Queries the document for a descendent that matches the given query.
	/// Returns None if no descendent is found.
	fn query_opt(&self, query: impl AsRef<str>) -> anyhow::Result<Option<&kdl::KdlNode>>;
	/// Queries the document for a descendent that matches the given query.
	/// Returns an error if no descendent is found.
	fn query_req(&self, query: impl AsRef<str>) -> anyhow::Result<&kdl::KdlNode>;
	
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a bool, an error is returned.
	fn get_bool_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<bool>>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a i64, an error is returned.
	fn get_i64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<i64>>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a f64, an error is returned.
	fn get_f64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<f64>>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a string, an error is returned.
	fn get_str_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<&str>>;

	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, and error is returned.
	/// If the entry is not a bool, an error is returned.
	fn get_bool_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<bool>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a i64, an error is returned.
	fn get_i64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<i64>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a f64, an error is returned.
	fn get_f64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<f64>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a string, an error is returned.
	fn get_str_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<&str>;
}

pub trait DocumentQueryExt {
	fn as_document(&self) -> Option<&kdl::KdlDocument>;

	fn query_bool(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<bool> {
		let doc = self.as_document().ok_or(MissingChildren)?;
		query_type(doc, query, key, as_bool)
	}

	fn query_i64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<i64> {
		let doc = self.as_document().ok_or(MissingChildren)?;
		query_type(doc, query, key, as_i64)
	}

	fn query_f64(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<f64> {
		let doc = self.as_document().ok_or(MissingChildren)?;
		query_type(doc, query, key, as_f64)
	}

	fn query_str(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<&str> {
		let doc = self.as_document().ok_or(MissingChildren)?;
		query_type(doc, query, key, as_str)
	}

	fn query_bool_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<bool>> {
		let Some(doc) = self.as_document() else { return Ok(None); };
		query_type_opt(doc, query, key, as_bool)
	}

	fn query_i64_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<i64>> {
		let Some(doc) = self.as_document() else { return Ok(None); };
		query_type_opt(doc, query, key, as_i64)
	}

	fn query_f64_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<f64>> {
		let Some(doc) = self.as_document() else { return Ok(None); };
		query_type_opt(doc, query, key, as_f64)
	}

	fn query_str_opt(
		&self,
		query: impl kdl::IntoKdlQuery + Clone,
		key: impl Into<kdl::NodeKey> + Clone,
	) -> anyhow::Result<Option<&str>> {
		let Some(doc) = self.as_document() else { return Ok(None); };
		query_type_opt(doc, query, key, as_str)
	}

	fn query_bool_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<bool>> + 'doc>> {
		let Some(doc) = self.as_document() else { return Ok(Box::new(Vec::new().into_iter())); };
		Ok(Box::new(query_type_all(doc, query, key, as_bool)?))
	}

	fn query_i64_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<i64>> + 'doc>> {
		let Some(doc) = self.as_document() else { return Ok(Box::new(Vec::new().into_iter())); };
		Ok(Box::new(query_type_all(doc, query, key, as_i64)?))
	}

	fn query_f64_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<f64>> + 'doc>> {
		let Some(doc) = self.as_document() else { return Ok(Box::new(Vec::new().into_iter())); };
		Ok(Box::new(query_type_all(doc, query, key, as_f64)?))
	}

	fn query_str_all<'doc>(
		&'doc self,
		query: impl kdl::IntoKdlQuery + Clone + 'doc,
		key: impl Into<kdl::NodeKey> + Clone + 'doc,
	) -> anyhow::Result<Box<dyn Iterator<Item = anyhow::Result<&str>> + 'doc>> {
		let Some(doc) = self.as_document() else { return Ok(Box::new(Vec::new().into_iter())); };
		Ok(Box::new(query_type_all(doc, query, key, as_str)?))
	}
}
pub trait NodeQueryExt {
	fn as_node(&self) -> &kdl::KdlNode;

	fn query_req(
		&self,
		query: impl kdl::IntoKdlQuery + Clone + std::fmt::Debug,
	) -> anyhow::Result<&kdl::KdlNode> {
		self.as_node().query(query.clone())?.ok_or(
			GeneralError(format!(
				"Missing child {:?} in node {:?}",
				query,
				self.as_node()
			))
			.into(),
		)
	}

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
			doc.query(query)?
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
	let Some(value) = doc.query_get(query.clone(), key)? else { return Ok(None); };
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
	fn as_document(&self) -> Option<&kdl::KdlDocument> {
		Some(&self)
	}
}
impl NodeQueryExt for kdl::KdlNode {
	fn as_node(&self) -> &kdl::KdlNode {
		self
	}
}
impl DocumentQueryExt for kdl::KdlNode {
	fn as_document(&self) -> Option<&kdl::KdlDocument> {
		self.children()
	}
}

#[derive(thiserror::Error, Debug)]
#[error("Document query requires children, but none are present.")]
struct MissingChildren;

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
			parentnode {
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
		assert_eq!(doc.query_bool("scope() > nodebool", 0)?, true);
		Ok(())
	}

	#[test]
	fn query_i64() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_i64("scope() > nodeint", 0)?, 42i64);
		Ok(())
	}

	#[test]
	fn query_float() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_f64("scope() > nodefloat", "prop")?, 10.5f64);
		Ok(())
	}

	#[test]
	fn query_str() -> anyhow::Result<()> {
		let doc = document()?;
		assert_eq!(doc.query_str("parentnode > nodestr", 0)?, "some string");
		Ok(())
	}
}
