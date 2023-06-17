mod as_kdl;
pub use as_kdl::*;
mod from_kdl;
pub use from_kdl::*;

#[derive(thiserror::Error, Debug)]
#[error("Entry \"{0}\" is missing a type identifier")]
pub struct EntryMissingType(pub kdl::KdlEntry);

/// The kdl value did not match the expected type.
#[derive(thiserror::Error, Debug)]
#[error("Invalid value {0:?}, was expecting a {1}")]
pub struct InvalidValueType(pub kdl::KdlValue, pub &'static str);

/// The node is missing an entry that was required.
#[derive(thiserror::Error, Debug)]
pub struct EntryMissing(pub kdl::KdlNode, pub kdl::NodeKey);
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
#[error("Query for {1:?} does not exist in {0}")]
pub struct QueryMissing(pub kdl::KdlDocument, pub String);

#[derive(thiserror::Error, Debug)]
#[error("Expected node to have children, but none are present in {0}")]
pub struct NoChildren(pub kdl::KdlNode);

#[derive(thiserror::Error, Debug)]
pub enum RequiredValue {
	#[error(transparent)]
	InvalidValueType(#[from] InvalidValueType),
	#[error(transparent)]
	EntryMissing(#[from] EntryMissing),
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
	#[error(transparent)]
	InvalidQuery(#[from] kdl::KdlError),
	#[error(transparent)]
	QueryMissing(#[from] QueryMissing),
	#[error(transparent)]
	InvalidValueType(#[from] InvalidValueType),
	#[error(transparent)]
	EntryMissing(#[from] EntryMissing),
	#[error(transparent)]
	NoChildren(#[from] NoChildren),
}
impl From<RequiredValue> for QueryError {
	fn from(value: RequiredValue) -> Self {
		match value {
			RequiredValue::InvalidValueType(err) => Self::InvalidValueType(err),
			RequiredValue::EntryMissing(err) => Self::EntryMissing(err),
		}
	}
}

pub trait ValueExt {
	/// Returns the value of the entry.
	/// If the value is not a bool, an error is returned.
	fn as_bool_req(&self) -> Result<bool, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a i64, an error is returned.
	fn as_i64_req(&self) -> Result<i64, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a f64, an error is returned.
	fn as_f64_req(&self) -> Result<f64, InvalidValueType>;
	/// Returns the value of the entry.
	/// If the value is not a string, an error is returned.
	fn as_str_req(&self) -> Result<&str, InvalidValueType>;
}
impl ValueExt for kdl::KdlValue {
	fn as_bool_req(&self) -> Result<bool, InvalidValueType> {
		Ok(self
			.as_bool()
			.ok_or(InvalidValueType(self.clone(), "bool"))?)
	}

	fn as_i64_req(&self) -> Result<i64, InvalidValueType> {
		Ok(self.as_i64().ok_or(InvalidValueType(self.clone(), "i64"))?)
	}

	fn as_f64_req(&self) -> Result<f64, InvalidValueType> {
		Ok(self.as_f64().ok_or(InvalidValueType(self.clone(), "f64"))?)
	}

	fn as_str_req(&self) -> Result<&str, InvalidValueType> {
		Ok(self
			.as_string()
			.ok_or(InvalidValueType(self.clone(), "string"))?)
	}
}

pub trait EntryExt {
	/// Returns the type of the entry.
	/// If the entry does not have a type, None is returned.
	fn type_opt(&self) -> Option<&str>;
	/// Returns the type of the entry.
	/// If the entry does not have a type, an error is returned.
	fn type_req(&self) -> Result<&str, EntryMissingType>;
}
impl EntryExt for kdl::KdlEntry {
	fn type_opt(&self) -> Option<&str> {
		self.ty().map(|id| id.value())
	}

	fn type_req(&self) -> Result<&str, EntryMissingType> {
		Ok(self.type_opt().ok_or(EntryMissingType(self.clone()))?)
	}
}
impl ValueExt for kdl::KdlEntry {
	fn as_bool_req(&self) -> Result<bool, InvalidValueType> {
		self.value().as_bool_req()
	}

	fn as_i64_req(&self) -> Result<i64, InvalidValueType> {
		self.value().as_i64_req()
	}

	fn as_f64_req(&self) -> Result<f64, InvalidValueType> {
		self.value().as_f64_req()
	}

	fn as_str_req(&self) -> Result<&str, InvalidValueType> {
		self.value().as_str_req()
	}
}

pub trait NodeExt {
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	fn entry_opt(&self, key: impl Into<kdl::NodeKey>) -> Option<&kdl::KdlEntry>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	fn entry_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&kdl::KdlEntry, EntryMissing>;

	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a bool, an error is returned.
	fn get_bool_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<bool>, InvalidValueType>;
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a i64, an error is returned.
	fn get_i64_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, InvalidValueType>;
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a f64, an error is returned.
	fn get_f64_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, InvalidValueType>;
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a string, an error is returned.
	fn get_str_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<&str>, InvalidValueType>;

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
	fn entry_opt(&self, key: impl Into<kdl::NodeKey>) -> Option<&kdl::KdlEntry> {
		self.entry(key)
	}

	fn entry_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&kdl::KdlEntry, EntryMissing> {
		let key = key.into();
		self.entry_opt(key.clone())
			.ok_or(EntryMissing(self.clone(), key))
	}

	fn get_bool_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<bool>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_bool_req()?))
	}

	fn get_i64_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_i64_req()?))
	}

	fn get_f64_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_f64_req()?))
	}

	fn get_str_opt(&self, key: impl Into<kdl::NodeKey>) -> Result<Option<&str>, InvalidValueType> {
		let Some(entry) = self.entry_opt(key) else { return Ok(None); };
		Ok(Some(entry.as_str_req()?))
	}

	fn get_bool_req(&self, key: impl Into<kdl::NodeKey>) -> Result<bool, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_bool_opt(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}

	fn get_i64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<i64, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_i64_opt(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}

	fn get_f64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<f64, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_f64_opt(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}

	fn get_str_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&str, RequiredValue> {
		let key = key.into();
		Ok(self
			.get_str_opt(key.clone())?
			.ok_or(EntryMissing(self.clone(), key))?)
	}
}

pub trait DocumentExt {
	/// Queries the document for a descendent that matches the given query.
	/// Returns None if no descendent is found.
	fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<&kdl::KdlNode>, kdl::KdlError>;
	/// Queries the document for a descendent that matches the given query.
	/// Returns an error if no descendent is found.
	fn query_req(&self, query: impl AsRef<str>) -> Result<&kdl::KdlNode, QueryError>;

	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a bool, an error is returned.
	fn query_bool_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, QueryError>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a i64, an error is returned.
	fn query_i64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<i64>, QueryError>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a f64, an error is returned.
	fn query_f64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<f64>, QueryError>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a string, an error is returned.
	fn query_str_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<&str>, QueryError>;

	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, and error is returned.
	/// If the entry is not a bool, an error is returned.
	fn query_bool_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<bool, QueryError>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a i64, an error is returned.
	fn query_i64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<i64, QueryError>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a f64, an error is returned.
	fn query_f64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<f64, QueryError>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a string, an error is returned.
	fn query_str_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<&str, QueryError>;

	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a bool, and error is returned.
	fn query_bool_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<bool>, QueryError>;
	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a i64, and error is returned.
	fn query_i64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<i64>, QueryError>;
	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a f64, and error is returned.
	fn query_f64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<f64>, QueryError>;
	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a string, and error is returned.
	fn query_str_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<&str>, QueryError>;
}

impl DocumentExt for kdl::KdlDocument {
	fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<&kdl::KdlNode>, kdl::KdlError> {
		self.query(query.as_ref())
	}

	fn query_req(&self, query: impl AsRef<str>) -> Result<&kdl::KdlNode, QueryError> {
		Ok(self
			.query_opt(query.as_ref())?
			.ok_or(QueryMissing(self.clone(), query.as_ref().to_owned()))?)
	}

	fn query_bool_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, QueryError> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_bool_opt(key)?)
	}

	fn query_i64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<i64>, QueryError> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_i64_opt(key)?)
	}

	fn query_f64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<f64>, QueryError> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_f64_opt(key)?)
	}

	fn query_str_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<&str>, QueryError> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_str_opt(key)?)
	}

	fn query_bool_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<bool, QueryError> {
		Ok(self.query_req(query)?.get_bool_req(key)?)
	}

	fn query_i64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<i64, QueryError> {
		Ok(self.query_req(query)?.get_i64_req(key)?)
	}

	fn query_f64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<f64, QueryError> {
		Ok(self.query_req(query)?.get_f64_req(key)?)
	}

	fn query_str_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<&str, QueryError> {
		Ok(self.query_req(query)?.get_str_req(key)?)
	}

	fn query_bool_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<bool>, QueryError> {
		let mut entries = Vec::new();
		let key = key.into();
		for node in self.query_all(query.as_ref())? {
			entries.push(node.get_bool_req(key.clone())?);
		}
		Ok(entries)
	}

	fn query_i64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<i64>, QueryError> {
		let mut entries = Vec::new();
		let key = key.into();
		for node in self.query_all(query.as_ref())? {
			entries.push(node.get_i64_req(key.clone())?);
		}
		Ok(entries)
	}

	fn query_f64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<f64>, QueryError> {
		let mut entries = Vec::new();
		let key = key.into();
		for node in self.query_all(query.as_ref())? {
			entries.push(node.get_f64_req(key.clone())?);
		}
		Ok(entries)
	}

	fn query_str_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<&str>, QueryError> {
		let mut entries = Vec::new();
		let key = key.into();
		for node in self.query_all(query.as_ref())? {
			entries.push(node.get_str_req(key.clone())?);
		}
		Ok(entries)
	}
}

impl DocumentExt for kdl::KdlNode {
	fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<&kdl::KdlNode>, kdl::KdlError> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_opt(query)
	}

	fn query_req(&self, query: impl AsRef<str>) -> Result<&kdl::KdlNode, QueryError> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_req(query)
	}

	fn query_bool_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, QueryError> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_bool_opt(query, key)
	}

	fn query_i64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<i64>, QueryError> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_i64_opt(query, key)
	}

	fn query_f64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<f64>, QueryError> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_f64_opt(query, key)
	}

	fn query_str_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<&str>, QueryError> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_str_opt(query, key)
	}

	fn query_bool_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<bool, QueryError> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_bool_req(query, key)
	}

	fn query_i64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<i64, QueryError> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_i64_req(query, key)
	}

	fn query_f64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<f64, QueryError> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_f64_req(query, key)
	}

	fn query_str_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<&str, QueryError> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_str_req(query, key)
	}

	fn query_bool_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<bool>, QueryError> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_bool_all(query, key)
	}

	fn query_i64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<i64>, QueryError> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_i64_all(query, key)
	}

	fn query_f64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<f64>, QueryError> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_f64_all(query, key)
	}

	fn query_str_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<&str>, QueryError> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_str_all(query, key)
	}
}

#[cfg(test)]
pub mod test_utils {
	use crate::kdl_ext::{AsKdl, FromKDL, NodeContext};

	pub fn from_doc_ctx<T: FromKDL>(
		name: &'static str,
		doc: &str,
		mut ctx: NodeContext,
	) -> anyhow::Result<T> {
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document
			.query(format!("scope() > {name}"))?
			.expect(&format!("missing {name} node"));
		T::from_kdl(node, &mut ctx)
	}

	pub fn from_doc<T: FromKDL>(name: &'static str, doc: &str) -> anyhow::Result<T> {
		from_doc_ctx::<T>(name, doc, NodeContext::default())
	}

	pub fn raw_doc(str: &'static str) -> String {
		use trim_margin::MarginTrimmable;
		str.trim_margin().unwrap()
	}

	pub fn as_doc(name: &'static str, data: &impl AsKdl) -> String {
		data.as_kdl().build(name).to_string()
	}
}
