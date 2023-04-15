//! A client-only IndexedDB is used to cache module data, so the application does not need to fetch all the contents of every module every time the app is opened. The database can be fully or partially refreshed as needed. Content is stored as raw kdl text in database entries, which can be parsed on the fly as content is needed for display or usage.
//! As of 2023.04.15, it is unclear to what degree this will replace system structures like `DnD5e`. There may be some data (like conditions) which need to stay in memory for easy access, while others (like items and spells) only need to be loaded when browsing content and relevant entries are loaded because they are a part of the character.
//! Each entry in the database is stored generically. It has a system id (e.g. `dnd5e`), a category (e.g. `item`, `spell`, `class`, `background`, etc.), and the kdl data associated with it. In the future, this could also include a `json` field for quickly converting between database and in-memory struct if kdl parsing proves to be too slow for on-the-fly usage.
use serde::{Deserialize, Serialize};

mod client;
pub use client::*;

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

	fn apply(&self, database: &idb::Database) -> Result<(), ClientError> {
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

	fn as_query(&self) -> Result<idb::Query, ClientError> {
		query([&self.system, &self.category])
	}
}
