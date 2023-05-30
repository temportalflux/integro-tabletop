use std::sync::Arc;

use super::Record;

pub mod entry;
pub use entry::Entry;
pub mod module;
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

	pub fn write(&self) -> Result<idb::Transaction, super::Error> {
		Ok(self.0.transaction(
			&[Entry::store_id(), Module::store_id()],
			idb::TransactionMode::ReadWrite,
		)?)
	}

	pub fn read_entries(&self) -> Result<idb::Transaction, super::Error> {
		Ok(self.0.read_only::<Entry>()?)
	}

	pub fn write_entries(&self) -> Result<idb::Transaction, super::Error> {
		Ok(self.0.read_write::<Entry>()?)
	}

	pub fn read_modules(&self) -> Result<idb::Transaction, super::Error> {
		Ok(self.0.read_only::<Module>()?)
	}

	pub fn write_modules(&self) -> Result<idb::Transaction, super::Error> {
		Ok(self.0.read_write::<Module>()?)
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

	fn read_index<I: super::IndexType>(&self) -> Result<super::Index<I>, super::Error> {
		use super::{ObjectStoreExt, TransactionExt};
		let transaction = self.read_entries()?;
		let entries_store = transaction.object_store_of::<I::Record>()?;
		entries_store.index_of::<I>()
	}

	pub async fn query<Output>(
		self,
		criteria: Box<Criteria>,
		node_reg: Arc<crate::system::core::NodeRegistry>,
	) -> Result<QueryDeserialize<Output>, super::Error>
	where
		Output: crate::kdl_ext::KDLNode
			+ crate::kdl_ext::FromKDL
			+ Unpin
			+ crate::system::dnd5e::SystemComponent,
		Output::System: crate::system::core::System,
	{
		use crate::system::core::System;
		let idx_by_sys_cate = self.read_index::<entry::SystemCategory>();
		let index = entry::SystemCategory {
			system: Output::System::id().into(),
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
}

impl std::ops::Deref for Database {
	type Target = super::Client;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
