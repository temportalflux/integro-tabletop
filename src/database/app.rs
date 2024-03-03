use crate::{kdl_ext::NodeContext, system::core::ModuleId};
use database::{Error, Record, Transaction};
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

	fn read_index<I: database::IndexType>(&self) -> Result<database::Index<I>, Error> {
		use database::{ObjectStoreExt, TransactionExt};
		let transaction = self.read_entries()?;
		let entries_store = transaction.object_store_of::<I::Record>()?;
		Ok(entries_store.index_of::<I>()?)
	}

	pub async fn get<T>(&self, key: impl Into<wasm_bindgen::JsValue>) -> Result<Option<T>, Error>
	where
		T: Record + serde::de::DeserializeOwned,
	{
		self.0.get::<T>(key).await
	}

	pub async fn get_typed_entry<T>(
		&self,
		key: crate::system::core::SourceId,
		system_depot: crate::system::Depot,
		criteria: Option<Criteria>,
	) -> Result<Option<T>, FetchError>
	where
		T: kdlize::NodeId
			+ kdlize::FromKdl<crate::kdl_ext::NodeContext>
			+ crate::system::dnd5e::SystemComponent
			+ Unpin,
	{
		use crate::system::core::System;
		let Some(entry) = self.get::<Entry>(key.to_string()).await? else {
			return Ok(None);
		};
		if let Some(criteria) = criteria {
			if !criteria.is_relevant(&entry.metadata) {
				return Ok(None);
			}
		}
		// Parse the entry's kdl string:
		// kdl string to document
		let document = entry.kdl.parse::<kdl::KdlDocument>()?;
		// document to node
		let node = match document.nodes().len() {
			1 => &document.nodes()[0],
			0 => return Err(FetchError::EmptyDocument),
			_ => return Err(FetchError::TooManyDocNodes(entry.kdl.clone())),
		};
		// node to value based on the expected type
		let node_reg = {
			let system_reg = system_depot
				.get(crate::system::dnd5e::DnD5e::id())
				.expect("Missing system {system:?} in depot");
			system_reg.node()
		};
		let ctx = crate::kdl_ext::NodeContext::new(Arc::new(entry.source_id(true)), node_reg);
		let Ok(value) = T::from_kdl(&mut crate::kdl_ext::NodeReader::new_root(node, ctx)) else {
			return Err(FetchError::FailedToParse(node.to_string(), T::id()));
		};
		Ok(Some(value))
	}

	pub async fn query_entries(
		&self,
		system: impl Into<String>,
		category: impl Into<String>,
		criteria: Option<Box<Criteria>>,
	) -> Result<Query, Error> {
		let idx_by_sys_cate = self.read_index::<entry::SystemCategory>();
		let index = entry::SystemCategory {
			system: system.into(),
			category: category.into(),
		};
		let cursor = idx_by_sys_cate?.open_cursor(Some(&index)).await?;
		Ok(Query { cursor, criteria })
	}

	pub async fn query_typed<Output>(
		self,
		system: impl Into<String>,
		system_depot: crate::system::Depot,
		criteria: Option<Box<Criteria>>,
	) -> Result<QueryDeserialize<Output>, Error>
	where
		Output: kdlize::NodeId + kdlize::FromKdl<NodeContext> + crate::system::dnd5e::SystemComponent + Unpin,
	{
		let system = system.into();
		let node_reg = {
			let system_reg = system_depot.get(&system).expect("Missing system {system:?} in depot");
			system_reg.node()
		};
		let idx_by_sys_cate = self.read_index::<entry::SystemCategory>();
		let index = entry::SystemCategory {
			system,
			category: Output::id().into(),
		};
		let cursor = idx_by_sys_cate?.open_cursor(Some(&index)).await?;
		let query_typed = QueryDeserialize::<Output> {
			db: self,
			query: Query { cursor, criteria },
			node_reg,
			marker: Default::default(),
		};
		Ok(query_typed)
	}

	pub async fn query_modules(self, system: Option<std::borrow::Cow<'_, str>>) -> Result<Vec<Module>, Error> {
		use database::{ObjectStoreExt, TransactionExt};
		use futures_util::StreamExt;
		let transaction = self.read_modules()?;
		let entries_store = transaction.object_store_of::<Module>()?;
		let idx_system = entries_store.index_of::<module::System>()?;
		let index = system.map(|system_id| module::System {
			system: system_id.into_owned(),
		});
		let mut cursor = idx_system.open_cursor(index.as_ref()).await?;
		let mut items = Vec::new();
		while let Some(item) = cursor.next().await {
			items.push(item);
		}
		Ok(items)
	}

	pub async fn query_entries_in(
		entry_store: &idb::ObjectStore,
		module_id: &ModuleId,
	) -> Result<database::Cursor<Entry>, Error> {
		use database::ObjectStoreExt;
		let idx_module = entry_store.index_of::<entry::Module>()?;
		Ok(idx_module
			.open_cursor(Some(&entry::Module {
				module: module_id.to_string(),
			}))
			.await?)
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
