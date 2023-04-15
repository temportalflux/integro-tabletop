use crate::utility::PinFutureLifetimeNoSend;
use wasm_bindgen::JsValue;

use super::{Error, Index, IndexType, Record};

pub trait ObjectStoreExt {
	fn put_record<'store, V>(
		&'store self,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), Error>>
	where
		V: Record;

	fn put_record_with_key<'store, K, V>(
		&'store self,
		key: &'store K,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), Error>>
	where
		K: 'static + wasm_bindgen::JsCast,
		JsValue: From<&'store K>,
		V: Record;

	fn create_index_of<T: IndexType>(
		&self,
		params: Option<idb::IndexParams>,
	) -> Result<idb::Index, idb::Error>;
	fn index_of<T: IndexType>(&self) -> Result<Index<T>, Error>;
}

impl ObjectStoreExt for idb::ObjectStore {
	fn put_record<'store, V>(
		&'store self,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), Error>>
	where
		V: Record,
	{
		Box::pin(async move {
			let value = record.as_value()?;
			let _ = self.put(&value, None).await?;
			Ok(())
		})
	}

	fn put_record_with_key<'store, K, V>(
		&'store self,
		key: &'store K,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), Error>>
	where
		K: 'static + wasm_bindgen::JsCast,
		JsValue: From<&'store K>,
		V: Record,
	{
		Box::pin(async move {
			let value = record.as_value()?;
			let key = key.into();
			let _ = self.put(&value, Some(&key)).await?;
			Ok(())
		})
	}

	fn create_index_of<T: IndexType>(
		&self,
		params: Option<idb::IndexParams>,
	) -> Result<idb::Index, idb::Error> {
		self.create_index(T::name(), T::key_path(), params)
	}

	fn index_of<T: IndexType>(&self) -> Result<Index<T>, Error> {
		Ok(Index::<T>::from(self.index(T::name())?))
	}
}

pub trait QueryExt {
	fn from_items<T: Into<JsValue>, const N: usize>(items: [T; N]) -> Result<idb::Query, Error>;
}
impl QueryExt for idb::Query {
	fn from_items<T: Into<JsValue>, const N: usize>(items: [T; N]) -> Result<idb::Query, Error> {
		if items.len() == 1 {
			let t_val = items.into_iter().next().unwrap();
			Ok(idb::Query::Key(t_val.into()))
		} else {
			let values = js_sys::Array::new_with_length(items.len() as u32);
			for (idx, t_val) in items.into_iter().enumerate() {
				values.set(idx as u32, t_val.into());
			}
			Ok(idb::Query::KeyRange(idb::KeyRange::only(&values)?))
		}
	}
}

pub trait TransactionExt {
	fn object_store_of<T: Record>(&self) -> Result<idb::ObjectStore, idb::Error>;
}
impl TransactionExt for idb::Transaction {
	fn object_store_of<T: Record>(&self) -> Result<idb::ObjectStore, idb::Error> {
		self.object_store(T::store_id())
	}
}
