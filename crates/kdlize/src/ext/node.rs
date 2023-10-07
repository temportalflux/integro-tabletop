use crate::{error::{InvalidValueType, MissingEntryValue, Error}, ext::ValueExt};

pub trait NodeExt {
	/// The node is searched for an entry which matches the given key. If no entry is found, None is returned.
	fn entry_opt(&self, key: impl Into<kdl::NodeKey>) -> Option<&kdl::KdlEntry>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	fn entry_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&kdl::KdlEntry, MissingEntryValue>;

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
	fn get_bool_req(&self, key: impl Into<kdl::NodeKey>) -> Result<bool, Error>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a i64, an error is returned.
	fn get_i64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<i64, Error>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a f64, an error is returned.
	fn get_f64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<f64, Error>;
	/// The node is searched for an entry which matches the given key. If no entry is found, an error is returned.
	/// If the entry is not a string, an error is returned.
	fn get_str_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&str, Error>;
}

impl NodeExt for kdl::KdlNode {
	fn entry_opt(&self, key: impl Into<kdl::NodeKey>) -> Option<&kdl::KdlEntry> {
		self.entry(key)
	}

	fn entry_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&kdl::KdlEntry, MissingEntryValue> {
		let key = key.into();
		self.entry_opt(key.clone())
			.ok_or(MissingEntryValue(self.clone(), key))
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

	fn get_bool_req(&self, key: impl Into<kdl::NodeKey>) -> Result<bool, Error> {
		let key = key.into();
		Ok(self
			.get_bool_opt(key.clone())?
			.ok_or(MissingEntryValue(self.clone(), key))?)
	}

	fn get_i64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<i64, Error> {
		let key = key.into();
		Ok(self
			.get_i64_opt(key.clone())?
			.ok_or(MissingEntryValue(self.clone(), key))?)
	}

	fn get_f64_req(&self, key: impl Into<kdl::NodeKey>) -> Result<f64, Error> {
		let key = key.into();
		Ok(self
			.get_f64_opt(key.clone())?
			.ok_or(MissingEntryValue(self.clone(), key))?)
	}

	fn get_str_req(&self, key: impl Into<kdl::NodeKey>) -> Result<&str, Error> {
		let key = key.into();
		Ok(self
			.get_str_opt(key.clone())?
			.ok_or(MissingEntryValue(self.clone(), key))?)
	}
}
