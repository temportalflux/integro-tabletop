use super::{Error, MissingVersion, Schema};
use idb::VersionChangeEvent;

/// A connection to a local IndexedDB database.
pub struct Client(idb::Database);

impl Client {
	pub async fn open<V>(name: &str) -> Result<Self, Error>
	where
		V: 'static + Schema + TryFrom<u32, Error = MissingVersion>,
	{
		let factory = idb::Factory::new()?;
		let mut request = factory.open(name, Some(V::latest()))?;
		// called when the database is being created for the first time, or when the version is old
		request.on_upgrade_needed(|event| {
			if let Err(err) = Self::upgrade_database::<V>(&event) {
				let old = event.old_version().unwrap();
				let new = event
					.new_version()
					.unwrap()
					.map(|v| format!(" to v{v}"))
					.unwrap_or_default();
				log::error!(target: "database::client", "Failed to upgrade database from v{old}{new}: {err:?}");
			}
		});
		let database = request.await?;
		Ok(Self(database))
	}

	fn upgrade_database<V>(event: &VersionChangeEvent) -> Result<(), Error>
	where
		V: 'static + Schema + TryFrom<u32, Error = MissingVersion>,
	{
		let database = event.database()?;
		// This is always 0 for database initialization, and is otherwise the previous version.
		let old_version = event.old_version()?;
		// I've never seen this be None in practice.
		let Some(new_version) = event.new_version()? else {
			return Ok(());
		};
		// Even if we are initializing fresh, we need to step through all of the versions
		for version in (old_version + 1)..=new_version {
			let schema = V::try_from(version)?;
			schema.apply(&database)?;
		}
		Ok(())
	}
}

impl Drop for Client {
	fn drop(&mut self) {
		self.0.close();
	}
}

impl Client {
	pub fn read_only<T: super::Record>(&self) -> Result<idb::Transaction, idb::Error> {
		self.0
			.transaction(&[T::store_id()], idb::TransactionMode::ReadOnly)
	}

	pub fn read_write<T: super::Record>(&self) -> Result<idb::Transaction, idb::Error> {
		self.0
			.transaction(&[T::store_id()], idb::TransactionMode::ReadWrite)
	}
}

impl std::ops::Deref for Client {
	type Target = idb::Database;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
