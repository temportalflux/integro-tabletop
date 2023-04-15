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

	fn index_of<T: IndexType>(&self) -> Result<Index<T>, Error> {
		Ok(Index::<T>::from(self.index(T::name())?))
	}
}
