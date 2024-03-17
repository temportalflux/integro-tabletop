use crate::system::Block;
use database::{Error, Record, Transaction};
use futures_util::future::LocalBoxFuture;

pub mod entry;
pub use entry::Entry;
pub mod module;
pub use module::Module;
mod query;
pub use query::*;
mod schema;
pub use schema::*;

#[derive(Clone, PartialEq)]
pub struct Database(database::Client);

impl Database {
	pub async fn open() -> Result<Self, Error> {
		let client = database::Client::open::<SchemaVersion>("tabletop-tools").await?;
		Ok(Self(client))
	}

	pub fn write(&self) -> Result<Transaction, Error> {
		self.0.transaction(
			&[Entry::store_id(), Module::store_id()],
			idb::TransactionMode::ReadWrite,
		)
	}

	pub fn read(&self) -> Result<Transaction, Error> {
		self.0
			.transaction(&[Entry::store_id(), Module::store_id()], idb::TransactionMode::ReadOnly)
	}

	pub fn read_entries(&self) -> Result<Transaction, Error> {
		self.0.read_only::<Entry>()
	}

	pub fn write_entries(&self) -> Result<Transaction, Error> {
		self.0.read_write::<Entry>()
	}

	pub fn read_modules(&self) -> Result<Transaction, Error> {
		self.0.read_only::<Module>()
	}

	pub fn write_modules(&self) -> Result<Transaction, Error> {
		self.0.read_write::<Module>()
	}

	pub async fn clear(&self) -> Result<(), Error> {
		use database::TransactionExt;
		let transaction = self.write()?;
		transaction.object_store_of::<Module>()?.clear()?.await?;
		transaction.object_store_of::<Entry>()?.clear()?.await?;
		transaction.commit().await?;
		Ok(())
	}

	pub async fn get<T>(&self, key: impl Into<wasm_bindgen::JsValue>) -> Result<Option<T>, Error>
	where
		T: Record + serde::de::DeserializeOwned,
	{
		self.0.get::<T>(key).await
	}

	pub async fn get_typed_entry<T>(
		&self,
		key: crate::system::SourceId,
		system_depot: crate::system::Registry,
		criteria: Option<Criteria>,
	) -> Result<Option<T>, FetchError>
	where
		T: Block + Unpin + 'static,
	{
		let query = Query::<Entry>::single(&self, &key).await?;
		let query = query.apply_opt(criteria, Query::filter_by);
		let mut query = query.parse_as::<T>(&system_depot);
		let Some((_entry, typed)) = query.next().await else {
			return Ok(None);
		};
		Ok(Some(typed))
	}

	pub async fn query_typed<Output>(
		self,
		system: impl Into<String>,
		system_depot: crate::system::Registry,
		criteria: Option<Box<Criteria>>,
	) -> Result<futures::stream::LocalBoxStream<'static, (Entry, Output)>, Error>
	where
		Output: Block + Unpin + 'static,
	{
		let index = entry::EntryInSystemWithType::new::<Output>(system);
		let query = Query::<Entry>::subset(&self, Some(index)).await?;
		let query = query.apply_opt(criteria.map(|c| *c), Query::filter_by);
		let query = query.parse_as::<Output>(&system_depot);
		Ok(query.into_inner())
	}

	pub async fn mutate<F>(&self, fn_transaction: F) -> Result<(), Error>
	where
		F: FnOnce(&database::Transaction) -> LocalBoxFuture<'_, Result<(), Error>>,
	{
		let transaction = self.write()?;
		fn_transaction(&transaction).await?;
		transaction.commit().await?;
		Ok(())
	}
}

impl std::ops::Deref for Database {
	type Target = database::Client;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum FetchError {
	#[error(transparent)]
	FindEntry(#[from] Error),
	#[error(transparent)]
	InvalidDocument(#[from] kdl::KdlError),
	#[error("Entry document is empty")]
	EmptyDocument,
	#[error("Entry document has too many nodes (should only be 1 per entry): {0:?}")]
	TooManyDocNodes(String),
	#[error("Failed to parse node as a {1:?}: {0:?}")]
	FailedToParse(String, &'static str),
}
