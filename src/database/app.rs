use crate::system::{Block, System};
use database::{Error, IndexType, Record, Transaction};
use futures::StreamExt;
use futures_util::future::LocalBoxFuture;
use std::sync::Arc;

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
		T: Block + Unpin,
	{
		let query = QuerySingle::<Entry>::from(key.to_string());
		let Some(entry) = query.execute(&self).await? else {
			return Ok(None);
		};
		if let Some(criteria) = criteria {
			if !criteria.is_relevant(&entry.metadata) {
				return Ok(None);
			}
		}
		let Some(typed) = entry.parse_kdl::<T>({
			let system_reg = system_depot
				.get(crate::system::dnd5e::DnD5e::id())
				.expect("Missing system {system:?} in depot");
			system_reg.node()
		}) else {
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
		let system = system.into();
		let node_reg = {
			let system_reg = system_depot.get(&system).expect("Missing system {system:?} in depot");
			system_reg.node()
		};
		let index = entry::SystemCategory {
			system,
			category: Output::id().into(),
		};
		self.query_index_typed(node_reg, index, criteria).await
	}

	pub async fn query_index_typed<Output, Index>(
		self,
		node_registry: Arc<crate::system::generics::Registry>,
		index: Index,
		criteria: Option<Box<Criteria>>,
	) -> Result<futures::stream::LocalBoxStream<'static, (Entry, Output)>, Error>
	where
		Output: Block + Unpin + 'static,
		Index: IndexType<Record = Entry>,
	{
		let query = QuerySubset::from(Some(index));
		let cursor = query.execute(&self).await?;
		let cursor = match criteria {
			None => cursor.boxed_local(),
			Some(criteria) => cursor.filter(criteria.into_predicate()).boxed_local(),
		};
		let cursor = cursor.parse_as::<Output>(node_registry);
		Ok(cursor)
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
