use database::{MissingVersion, ObjectStoreExt, Record, Schema};

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

	fn apply(&self, database: &idb::Database, _transaction: Option<&idb::Transaction>) -> Result<(), idb::Error> {
		match self {
			Self::Version1 => {
				// Create modules table
				{
					use crate::database::module::{Module, System};
					let mut params = idb::ObjectStoreParams::new();
					params.auto_increment(true);
					params.key_path(Some(idb::KeyPath::new_single("name")));
					let store = database.create_object_store(Module::store_id(), params)?;
					store.create_index_of::<System>(Some({
						let mut params = idb::IndexParams::new();
						params.multi_entry(true);
						params
					}))?;
				}
				// Create entries table
				{
					use crate::database::entry::{Entry, Module, ModuleSystem, System, SystemCategory};
					let mut params = idb::ObjectStoreParams::new();
					params.auto_increment(true);
					params.key_path(Some(idb::KeyPath::new_single("id")));
					let store = database.create_object_store(Entry::store_id(), params)?;
					store.create_index_of::<Module>(None)?;
					store.create_index_of::<ModuleSystem>(None)?;
					store.create_index_of::<System>(None)?;
					store.create_index_of::<SystemCategory>(None)?;
				}
			}
		}
		Ok(())
	}
}
