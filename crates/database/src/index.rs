use super::{Cursor, Error};
use serde::Deserialize;

pub struct Index<T: IndexType>(idb::Index, std::marker::PhantomData<T>);

impl<T: IndexType> From<idb::Index> for Index<T> {
	fn from(value: idb::Index) -> Self {
		Self(value, Default::default())
	}
}

impl<T: IndexType> Index<T> {
	pub async fn get<'index>(&'index self, params: &'index T) -> Result<Option<T::Record>, Error>
	where
		T::Record: for<'de> Deserialize<'de>,
	{
		match self.0.get(params.as_query()?).await? {
			Some(js_value) => Ok(Some(serde_wasm_bindgen::from_value::<T::Record>(js_value)?)),
			None => Ok(None),
		}
	}

	pub async fn get_all<'index>(&'index self, params: &'index T, limit: Option<u32>) -> Result<Vec<T::Record>, Error>
	where
		T::Record: for<'de> Deserialize<'de>,
	{
		let js_values = self.0.get_all(Some(params.as_query()?), limit).await?;
		let mut values = Vec::with_capacity(js_values.len());
		for js_value in js_values {
			values.push(serde_wasm_bindgen::from_value::<T::Record>(js_value)?);
		}
		Ok(values)
	}

	pub async fn open_cursor(&self, params: Option<&T>) -> Result<Cursor<T::Record>, idb::Error>
	where
		T::Record: for<'de> Deserialize<'de>,
	{
		let query = match params {
			Some(params) => Some(params.as_query()?),
			None => None,
		};
		let cursor = self.0.open_cursor(query, None).await?;
		let cursor = Cursor::<T::Record>::new(cursor);
		Ok(cursor)
	}
}

pub trait IndexType {
	type Record: super::Record;

	fn name() -> &'static str;
	fn keys() -> &'static [&'static str];
	fn as_query(&self) -> Result<idb::Query, idb::Error>;

	fn key_path() -> idb::KeyPath {
		let keys = Self::keys();
		if keys.len() == 1 {
			idb::KeyPath::new_single(keys[0])
		} else {
			idb::KeyPath::new_array(keys.to_vec())
		}
	}
}
