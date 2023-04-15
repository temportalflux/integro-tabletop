use super::{query, Error, IndexType, MissingVersion, Record, Schema};
use serde::{Deserialize, Serialize};

pub static ENTRY_TABLE: &'static str = "entries";

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
	pub id: String,
	pub module: String,
	pub system: String,
	pub category: String,
	pub kdl: String,
}
impl Record for Entry {}

/// The schema for the `tabletop-tools` client database.
/// Use with `Client::open`.
pub enum SchemaVersion {
	Version1 = 1,
}

impl TryFrom<u32> for SchemaVersion {
	type Error = MissingVersion;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		match value {
			1 => Ok(Self::Version1),
			_ => Err(MissingVersion(value)),
		}
	}
}

impl Schema for SchemaVersion {
	fn latest() -> u32 {
		Self::Version1 as u32
	}

	fn apply(&self, database: &idb::Database) -> Result<(), Error> {
		let entries_store = {
			let mut params = idb::ObjectStoreParams::new();
			params.auto_increment(true);
			params.key_path(Some(idb::KeyPath::new_single("id")));
			database.create_object_store(ENTRY_TABLE, params)?
		};
		entries_store.create_index(SystemCategory::name(), SystemCategory::key_path(), None)?;
		Ok(())
	}
}

pub struct SystemCategory {
	pub system: String,
	pub category: String,
}
impl IndexType for SystemCategory {
	fn name() -> &'static str {
		"system_category"
	}

	fn keys() -> &'static [&'static str] {
		&["system", "category"]
	}

	fn as_query(&self) -> Result<idb::Query, Error> {
		query([&self.system, &self.category])
	}
}
