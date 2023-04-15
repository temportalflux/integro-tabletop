use super::Record;

pub mod entry;
pub use entry::Entry;
pub mod module;
pub use module::Module;
mod schema;
pub use schema::*;

#[derive(Clone, PartialEq)]
pub struct Database(super::Client);

impl Database {
	pub async fn open() -> Result<Self, super::Error> {
		let client = super::Client::open::<SchemaVersion>("tabletop-tools").await?;
		Ok(Self(client))
	}

	pub fn write(&self) -> Result<idb::Transaction, idb::Error> {
		self.0.transaction(
			&[Entry::store_id(), Module::store_id()],
			idb::TransactionMode::ReadWrite,
		)
	}

	pub fn read_entries(&self) -> Result<idb::Transaction, idb::Error> {
		self.0.read_only::<Entry>()
	}

	pub fn write_entries(&self) -> Result<idb::Transaction, idb::Error> {
		self.0.read_write::<Entry>()
	}

	pub fn read_modules(&self) -> Result<idb::Transaction, idb::Error> {
		self.0.read_only::<Module>()
	}

	pub fn write_modules(&self) -> Result<idb::Transaction, idb::Error> {
		self.0.read_write::<Module>()
	}
}

impl std::ops::Deref for Database {
	type Target = super::Client;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
