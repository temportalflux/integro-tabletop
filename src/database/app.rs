use std::sync::Arc;

use super::Record;

pub mod entry;
pub use entry::Entry;
pub mod module;
use futures_util::future::LocalBoxFuture;
pub use module::Module;
mod query;
pub use query::*;
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
		Ok(self.0.transaction(
			&[Entry::store_id(), Module::store_id()],
			idb::TransactionMode::ReadWrite,
		)?)
	}

	pub fn read_entries(&self) -> Result<idb::Transaction, idb::Error> {
		Ok(self.0.read_only::<Entry>()?)
	}

	pub fn write_entries(&self) -> Result<idb::Transaction, idb::Error> {
		Ok(self.0.read_write::<Entry>()?)
	}

	pub fn read_modules(&self) -> Result<idb::Transaction, idb::Error> {
		Ok(self.0.read_only::<Module>()?)
	}

	pub fn write_modules(&self) -> Result<idb::Transaction, idb::Error> {
		Ok(self.0.read_write::<Module>()?)
	}

	pub async fn clear(&self) -> Result<(), idb::Error> {
		use crate::database::TransactionExt;
		let transaction = self.write()?;
		transaction.object_store_of::<Module>()?.clear().await?;
		transaction.object_store_of::<Entry>()?.clear().await?;
		transaction.commit().await?;
		Ok(())
	}

	fn read_index<I: super::IndexType>(&self) -> Result<super::Index<I>, idb::Error> {
		use super::{ObjectStoreExt, TransactionExt};
		let transaction = self.read_entries()?;
		let entries_store = transaction.object_store_of::<I::Record>()?;
		entries_store.index_of::<I>()
	}

	pub async fn get<T>(
		&self,
		key: impl Into<wasm_bindgen::JsValue>,
	) -> Result<Option<T>, super::Error>
	where
		T: Record + serde::de::DeserializeOwned,
	{
		use super::{ObjectStoreExt, TransactionExt};
		let transaction = self.0.read_only::<T>()?;
		let store = transaction.object_store_of::<T>()?;
		Ok(store.get_record(key).await?)
	}

	pub async fn get_typed_entry<T>(
		&self,
		key: crate::system::core::SourceId,
		system_depot: crate::system::Depot,
		criteria: Option<Criteria>,
	) -> Result<Option<T>, FetchError>
	where
		T: crate::kdl_ext::KDLNode
			+ crate::kdl_ext::FromKDL
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
	) -> Result<Query, super::Error> {
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
	) -> Result<QueryDeserialize<Output>, idb::Error>
	where
		Output: crate::kdl_ext::KDLNode
			+ crate::kdl_ext::FromKDL
			+ crate::system::dnd5e::SystemComponent
			+ Unpin,
	{
		let system = system.into();
		let node_reg = {
			let system_reg = system_depot
				.get(&system)
				.expect("Missing system {system:?} in depot");
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

	pub async fn mutate<F>(&self, fn_transaction: F) -> Result<(), super::Error>
	where
		F: FnOnce(&idb::Transaction) -> LocalBoxFuture<'_, Result<(), super::Error>>,
	{
		let transaction = self.write()?;
		fn_transaction(&transaction).await?;
		transaction.commit().await?;
		Ok(())
	}
}

impl std::ops::Deref for Database {
	type Target = super::Client;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum FetchError {
	#[error(transparent)]
	FindEntry(#[from] super::Error),
	#[error(transparent)]
	InvalidDocument(#[from] kdl::KdlError),
	#[error("Entry document is empty")]
	EmptyDocument,
	#[error("Entry document has too many nodes (should only be 1 per entry): {0:?}")]
	TooManyDocNodes(String),
	#[error("Failed to parse node as a {1:?}: {0:?}")]
	FailedToParse(String, &'static str),
}
