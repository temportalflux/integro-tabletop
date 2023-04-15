use super::{Cursor, Error};
use serde::Deserialize;

pub struct Index<T: IndexType>(idb::Index, std::marker::PhantomData<T>);

impl<T: IndexType> From<idb::Index> for Index<T> {
	fn from(value: idb::Index) -> Self {
		Self(value, Default::default())
	}
}

impl<T: IndexType> Index<T> {
	pub async fn get_all<'index, V>(
		&'index self,
		params: &'index T,
		limit: Option<u32>,
	) -> Result<Vec<V>, Error>
	where
		V: for<'de> Deserialize<'de>,
	{
		let js_values = self.0.get_all(Some(params.as_query()?), limit).await?;
		let mut values = Vec::with_capacity(js_values.len());
		for js_value in js_values {
			values.push(serde_wasm_bindgen::from_value::<V>(js_value)?);
		}
		Ok(values)
	}

	pub async fn open_cursor<V>(&self, params: Option<&T>) -> Result<Cursor<V>, Error>
	where
		V: for<'de> Deserialize<'de>,
	{
		let query = match params {
			Some(params) => Some(params.as_query()?),
			None => None,
		};
		let cursor = self.0.open_cursor(query, None).await?;
		let cursor = Cursor::<V>::new(cursor);
		Ok(cursor)
	}
}

pub trait IndexType {
	fn name() -> &'static str;
	fn keys() -> &'static [&'static str];
	fn as_query(&self) -> Result<idb::Query, Error>;

	fn key_path() -> idb::KeyPath {
		let keys = Self::keys();
		if keys.len() == 1 {
			idb::KeyPath::new_single(keys[0])
		} else {
			idb::KeyPath::new_array(keys.to_vec())
		}
	}
}
