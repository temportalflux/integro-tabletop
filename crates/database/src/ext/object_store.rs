use super::super::{Error, Index, IndexType, Record};
use futures_util::future::LocalBoxFuture;
use wasm_bindgen::JsValue;

pub trait ObjectStoreExt {
	fn get_record<'store, V>(
		&'store self,
		key: impl Into<JsValue> + 'store,
	) -> LocalBoxFuture<'store, Result<Option<V>, Error>>
	where
		V: Record + serde::de::DeserializeOwned;

	fn delete_record<'store>(
		&'store self,
		key: impl Into<JsValue> + 'store,
	) -> LocalBoxFuture<'store, Result<(), Error>>;

	fn add_record<'store, V>(&'store self, record: &'store V) -> LocalBoxFuture<'store, Result<(), Error>>
	where
		V: Record;

	fn put_record<'store, V>(&'store self, record: &'store V) -> LocalBoxFuture<'store, Result<(), Error>>
	where
		V: Record;

	fn put_record_with_key<'store, K, V>(
		&'store self,
		key: &'store K,
		record: &'store V,
	) -> LocalBoxFuture<'store, Result<(), Error>>
	where
		K: 'static + wasm_bindgen::JsCast,
		JsValue: From<&'store K>,
		V: Record;

	fn create_index_of<T: IndexType>(&self, params: Option<idb::IndexParams>) -> Result<idb::Index, idb::Error>;
	fn index_of<T: IndexType>(&self) -> Result<Index<T>, idb::Error>;
}

impl ObjectStoreExt for idb::ObjectStore {
	fn get_record<'store, V>(
		&'store self,
		key: impl Into<JsValue> + 'store,
	) -> LocalBoxFuture<'store, Result<Option<V>, Error>>
	where
		V: Record + serde::de::DeserializeOwned,
	{
		Box::pin(async move {
			let Some(record_js) = self.get(idb::Query::Key(key.into())).await? else {
				return Ok(None);
			};
			Ok(Some(serde_wasm_bindgen::from_value::<V>(record_js)?))
		})
	}

	fn delete_record<'store>(
		&'store self,
		key: impl Into<JsValue> + 'store,
	) -> LocalBoxFuture<'store, Result<(), Error>> {
		Box::pin(async move {
			self.delete(idb::Query::Key(key.into())).await?;
			Ok(())
		})
	}

	fn add_record<'store, V>(&'store self, record: &'store V) -> LocalBoxFuture<'store, Result<(), Error>>
	where
		V: Record,
	{
		Box::pin(async move {
			let key = record.key().map(|key| JsValue::from(key));
			let value = record.as_value()?;
			let _ = self.add(&value, key.as_ref()).await?;
			Ok(())
		})
	}

	fn put_record<'store, V>(&'store self, record: &'store V) -> LocalBoxFuture<'store, Result<(), Error>>
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
	) -> LocalBoxFuture<'store, Result<(), Error>>
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

	fn create_index_of<T: IndexType>(&self, params: Option<idb::IndexParams>) -> Result<idb::Index, idb::Error> {
		self.create_index(T::name(), T::key_path(), params)
	}

	fn index_of<T: IndexType>(&self) -> Result<Index<T>, idb::Error> {
		Ok(Index::<T>::from(self.index(T::name())?))
	}
}
