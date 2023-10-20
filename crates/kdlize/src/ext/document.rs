use crate::{
	error::{Error, InvalidQueryFormat, NoChildren, QueryMissing},
	ext::NodeExt,
};

pub trait DocumentQueryExt {
	/// Queries the document for a descendent that matches the given query.
	/// Returns None if no descendent is found.
	fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<&kdl::KdlNode>, InvalidQueryFormat>;
	/// Queries the document for a descendent that matches the given query.
	/// Returns an error if no descendent is found.
	fn query_req(&self, query: impl AsRef<str>) -> Result<&kdl::KdlNode, Error>;
}

pub trait DocumentExt {
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a bool, an error is returned.
	fn query_bool_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<bool>, Error>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a i64, an error is returned.
	fn query_i64_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, Error>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a f64, an error is returned.
	fn query_f64_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, Error>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, None is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, None is returned.
	/// If the entry is not a string, an error is returned.
	fn query_str_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<&str>, Error>;

	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, and error is returned.
	/// If the entry is not a bool, an error is returned.
	fn query_bool_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<bool, Error>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a i64, an error is returned.
	fn query_i64_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<i64, Error>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a f64, an error is returned.
	fn query_f64_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<f64, Error>;
	/// Queries the document for a descendent that matches the given query. If no descendent is found, an error is returned.
	/// The descedent is then searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a string, an error is returned.
	fn query_str_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<&str, Error>;

	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a bool, and error is returned.
	fn query_bool_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<bool>, Error>;
	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a i64, and error is returned.
	fn query_i64_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<i64>, Error>;
	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a f64, and error is returned.
	fn query_f64_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<f64>, Error>;
	/// Queries the document for all descendents that match the given query. If no descendents are found, and empty vec is returned.
	/// If any descendents which match the query do not have an entry at the given key, an error is returned.
	/// If the entry of any of those descendents is not a string, and error is returned.
	fn query_str_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<&str>, Error>;
}

impl DocumentQueryExt for kdl::KdlDocument {
	fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<&kdl::KdlNode>, InvalidQueryFormat> {
		self.query(query.as_ref()).map_err(|e| InvalidQueryFormat(e))
	}

	fn query_req(&self, query: impl AsRef<str>) -> Result<&kdl::KdlNode, Error> {
		Ok(self
			.query_opt(query.as_ref())?
			.ok_or(QueryMissing(self.clone(), query.as_ref().to_owned()))?)
	}
}

impl DocumentExt for kdl::KdlDocument {
	fn query_bool_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<bool>, Error> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_bool_opt(key)?)
	}

	fn query_i64_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, Error> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_i64_opt(key)?)
	}

	fn query_f64_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, Error> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_f64_opt(key)?)
	}

	fn query_str_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<&str>, Error> {
		let Some(node) = self.query_opt(query)? else { return Ok(None); };
		Ok(node.get_str_opt(key)?)
	}

	fn query_bool_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<bool, Error> {
		Ok(self.query_req(query)?.get_bool_req(key)?)
	}

	fn query_i64_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<i64, Error> {
		Ok(self.query_req(query)?.get_i64_req(key)?)
	}

	fn query_f64_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<f64, Error> {
		Ok(self.query_req(query)?.get_f64_req(key)?)
	}

	fn query_str_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<&str, Error> {
		Ok(self.query_req(query)?.get_str_req(key)?)
	}

	fn query_bool_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<bool>, Error> {
		let mut entries = Vec::new();
		let key = key.into();
		let iter = self.query_all(query.as_ref()).map_err(|e| InvalidQueryFormat(e))?;
		for node in iter {
			entries.push(node.get_bool_req(key.clone())?);
		}
		Ok(entries)
	}

	fn query_i64_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<i64>, Error> {
		let mut entries = Vec::new();
		let key = key.into();
		let iter = self.query_all(query.as_ref()).map_err(|e| InvalidQueryFormat(e))?;
		for node in iter {
			entries.push(node.get_i64_req(key.clone())?);
		}
		Ok(entries)
	}

	fn query_f64_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<f64>, Error> {
		let mut entries = Vec::new();
		let key = key.into();
		let iter = self.query_all(query.as_ref()).map_err(|e| InvalidQueryFormat(e))?;
		for node in iter {
			entries.push(node.get_f64_req(key.clone())?);
		}
		Ok(entries)
	}

	fn query_str_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<&str>, Error> {
		let mut entries = Vec::new();
		let key = key.into();
		let iter = self.query_all(query.as_ref()).map_err(|e| InvalidQueryFormat(e))?;
		for node in iter {
			entries.push(node.get_str_req(key.clone())?);
		}
		Ok(entries)
	}
}

impl DocumentQueryExt for kdl::KdlNode {
	fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<&kdl::KdlNode>, InvalidQueryFormat> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_opt(query)
	}

	fn query_req(&self, query: impl AsRef<str>) -> Result<&kdl::KdlNode, Error> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_req(query)
	}
}

impl DocumentExt for kdl::KdlNode {
	fn query_bool_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<bool>, Error> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_bool_opt(query, key)
	}

	fn query_i64_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<i64>, Error> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_i64_opt(query, key)
	}

	fn query_f64_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<f64>, Error> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_f64_opt(query, key)
	}

	fn query_str_opt(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Option<&str>, Error> {
		let Some(doc) = self.children() else { return Ok(None); };
		doc.query_str_opt(query, key)
	}

	fn query_bool_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<bool, Error> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_bool_req(query, key)
	}

	fn query_i64_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<i64, Error> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_i64_req(query, key)
	}

	fn query_f64_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<f64, Error> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_f64_req(query, key)
	}

	fn query_str_req(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<&str, Error> {
		let doc = self.children().ok_or(NoChildren(self.clone()))?;
		doc.query_str_req(query, key)
	}

	fn query_bool_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<bool>, Error> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_bool_all(query, key)
	}

	fn query_i64_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<i64>, Error> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_i64_all(query, key)
	}

	fn query_f64_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<f64>, Error> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_f64_all(query, key)
	}

	fn query_str_all(&self, query: impl AsRef<str>, key: impl Into<kdl::NodeKey>) -> Result<Vec<&str>, Error> {
		let Some(doc) = self.children() else { return Ok(Vec::new()); };
		doc.query_str_all(query, key)
	}
}
