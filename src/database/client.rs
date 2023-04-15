use crate::utility::PinFutureLifetimeNoSend;
use futures_util::Future;
use idb::VersionChangeEvent;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, task::Poll};
use wasm_bindgen::JsValue;

/// A connection to a local IndexedDB database.
pub struct Client(idb::Database);

impl Client {
	pub async fn open<V>(name: &str) -> Result<Self, ClientError>
	where
		V: 'static + Schema + TryFrom<u32, Error = MissingVersion>,
	{
		let factory = idb::Factory::new()?;
		let mut request = factory.open(name, Some(V::latest()))?;
		// called when the database is being created for the first time, or when the version is old
		request.on_upgrade_needed(|event| {
			if let Err(err) = Self::upgrade_database::<V>(&event) {
				let old = event.old_version().unwrap();
				let new = event
					.new_version()
					.unwrap()
					.map(|v| format!(" to v{v}"))
					.unwrap_or_default();
				log::error!(target: "database::client", "Failed to upgrade database from v{old}{new}: {err:?}");
			}
		});
		let database = request.await?;
		Ok(Self(database))
	}

	fn upgrade_database<V>(event: &VersionChangeEvent) -> Result<(), ClientError>
	where
		V: 'static + Schema + TryFrom<u32, Error = MissingVersion>,
	{
		let database = event.database()?;
		// This is always 0 for database initialization, and is otherwise the previous version.
		let old_version = event.old_version()?;
		// I've never seen this be None in practice.
		let Some(new_version) = event.new_version()? else {
			return Ok(());
		};
		// Even if we are initializing fresh, we need to step through all of the versions
		for version in (old_version + 1)..=new_version {
			let schema = V::try_from(version)?;
			schema.apply(&database)?;
		}
		Ok(())
	}
}
impl Drop for Client {
	fn drop(&mut self) {
		self.0.close();
	}
}
impl Client {
	pub fn transaction<T>(
		&self,
		store_names: &[T],
		mode: idb::TransactionMode,
	) -> Result<idb::Transaction, ClientError>
	where
		T: AsRef<str>,
	{
		Ok(self.0.transaction(store_names, mode)?)
	}
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
	#[error(transparent)]
	DatabaseError(#[from] idb::Error),
	#[error(transparent)]
	MissingSchemaVersion(#[from] MissingVersion),
	#[error(transparent)]
	FailedToSerialize(#[from] serde_wasm_bindgen::Error),
}

#[derive(thiserror::Error, Debug)]
#[error("Schema is missing version {0}.")]
pub struct MissingVersion(pub u32);

pub trait Schema {
	fn latest() -> u32;
	fn apply(&self, database: &idb::Database) -> Result<(), ClientError>;
}

pub trait Record: Serialize {
	fn as_value(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
		Ok(self.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
	}
}

pub trait ObjectStoreExt {
	fn put_record<'store, V>(
		&'store self,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), ClientError>>
	where
		V: Record;

	fn put_record_with_key<'store, K, V>(
		&'store self,
		key: &'store K,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), ClientError>>
	where
		K: 'static + wasm_bindgen::JsCast,
		JsValue: From<&'store K>,
		V: Record;

	fn index_of<T: IndexType>(&self) -> Result<Index<T>, ClientError>;
}
impl ObjectStoreExt for idb::ObjectStore {
	fn put_record<'store, V>(
		&'store self,
		record: &'store V,
	) -> PinFutureLifetimeNoSend<'store, Result<(), ClientError>>
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
	) -> PinFutureLifetimeNoSend<'store, Result<(), ClientError>>
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

	fn index_of<T: IndexType>(&self) -> Result<Index<T>, ClientError> {
		Ok(Index(self.index(T::name())?, Default::default()))
	}
}

pub struct Index<T: IndexType>(idb::Index, std::marker::PhantomData<T>);
impl<T: IndexType> Index<T> {
	pub async fn get_all<'index, V>(
		&'index self,
		params: &'index T,
		limit: Option<u32>,
	) -> Result<Vec<V>, ClientError>
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

	pub async fn open_cursor<V>(&self, params: Option<&T>) -> Result<Cursor<V>, ClientError>
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
	fn as_query(&self) -> Result<idb::Query, ClientError>;

	fn key_path() -> idb::KeyPath {
		let keys = Self::keys();
		if keys.len() == 1 {
			idb::KeyPath::new_single(keys[0])
		} else {
			idb::KeyPath::new_array(keys.to_vec())
		}
	}
}

pub fn query<T: Into<JsValue>, const N: usize>(items: [T; N]) -> Result<idb::Query, ClientError> {
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

/// Iterates over the contents of a cursor provided by one of the `open_cursor` functions.
/// You can iterate over it like an async iterator / stream:
/// ```no_run
/// while let Some(entry) = cursor.next().await {
///   // ...
/// }
/// ```
/// or manually iterate, granting access to functions to update or delete
/// the database entry the cursor is during iteration:
/// ```ignore
/// while let Some(entry) = cursor.value()? {
///   //let entry = cursor.update_value(new_value).await?;
///   //cursor.delete_value().await?;
///   cursor.advance().await?;
/// }
/// ```
pub struct Cursor<V> {
	cursor: Option<idb::Cursor>,
	marker: std::marker::PhantomData<V>,
	advance: Option<Pin<Box<dyn Future<Output = Result<idb::Cursor, idb::Error>>>>>,
}
impl<V> Cursor<V> {
	pub fn new(cursor: Option<idb::Cursor>) -> Self {
		Self {
			cursor,
			marker: Default::default(),
			advance: None,
		}
	}

	pub fn value(&self) -> Result<Option<V>, ClientError>
	where
		V: for<'de> Deserialize<'de>,
	{
		let Some(cursor) = &self.cursor else { return Ok(None); };
		let value = cursor.value()?;
		if value.is_null() {
			return Ok(None);
		}
		Ok(Some(serde_wasm_bindgen::from_value::<V>(value)?))
	}

	pub async fn advance(&mut self) -> Result<(), idb::Error> {
		if let Some(cursor) = &mut self.cursor {
			cursor.advance(1).await?;
		}
		Ok(())
	}

	pub async fn update_value(&self, new_value: &V) -> Result<Option<V>, ClientError>
	where
		V: Serialize + for<'de> Deserialize<'de>,
	{
		let Some(cursor) = &self.cursor else {
			return Ok(None);
		};
		let js_value = serde_wasm_bindgen::to_value(new_value)?;
		let js_value = cursor.update(&js_value).await?;
		Ok(Some(serde_wasm_bindgen::from_value(js_value)?))
	}

	pub async fn delete_value(&self) -> Result<(), idb::Error> {
		if let Some(cursor) = &self.cursor {
			cursor.delete().await?;
		}
		Ok(())
	}
}
impl<V> futures_util::stream::Stream for Cursor<V>
where
	V: for<'de> Deserialize<'de> + Unpin,
{
	type Item = V;

	fn poll_next(
		mut self: Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Option<Self::Item>> {
		loop {
			// Process any pending advancement future first.
			// If there is a future here, it means we are waiting for the underlying cursor
			// to finish advancing before finding the next value.
			if let Some(mut advance) = self.advance.take() {
				match advance.as_mut().poll(cx) {
					// the cursor is still advancing, poll the stream later
					Poll::Pending => {
						self.advance = Some(advance);
						return Poll::Pending;
					}
					// advancing found an error, lets assume its the end of stream
					Poll::Ready(Err(err)) => {
						log::error!(target: "cursor", "Failed to advance idb::Cursor: {err:?}");
						return Poll::Ready(None);
					}
					// the advancement has finished, we can resume the finding-of-next-value.
					Poll::Ready(Ok(cursor)) => {
						self.cursor = Some(cursor);
					}
				}
			}

			// There should ALWAYS be a cursor if we are not advancing and this stream was provided a cursor.
			// If there is no cursor, then one was not provided by one of the `open_cursor` functions, so the stream is empty.
			let Some(cursor) = self.cursor.take() else { return Poll::Ready(None); };

			// Cursor exists and there is probably a next value, lets find out.
			let value = match cursor.value() {
				Ok(value) => value,
				Err(err) => {
					log::error!(target: "cursor", "Failed to get next value in idb::Cursor: {err:?}");
					return Poll::Ready(None);
				}
			};
			// Value is empty, so we've reached end-of-stream.
			if value.is_null() {
				return Poll::Ready(None);
			}
			// Parse the valid JSValue as the desired struct type.
			let value = match serde_wasm_bindgen::from_value::<V>(value) {
				Ok(value) => value,
				Err(err) => {
					log::error!(target: "cursor", "Failed to parse database value: {err:?}");
					continue;
				}
			};
			// Prime the advance future for the next loop or next time this stream is polled.
			self.advance = Some(Box::pin(async move {
				// move the cursor in so this future can have a static lifetime
				let mut cursor = cursor;
				cursor.advance(1).await?;
				Ok(cursor)
			}));
			// Return the found value, while advancement run in the background.
			return Poll::Ready(Some(value));
		}
	}
}
