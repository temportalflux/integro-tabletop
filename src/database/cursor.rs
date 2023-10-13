use super::Error;
use futures_util::Future;
use serde::{Deserialize, Serialize};
use std::{pin::Pin, task::Poll};

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

	pub fn value(&self) -> Result<Option<V>, Error>
	where
		V: for<'de> Deserialize<'de>,
	{
		let Some(cursor) = &self.cursor else {
			return Ok(None);
		};
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

	pub async fn update_value(&self, new_value: &V) -> Result<Option<V>, Error>
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

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
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
			let Some(cursor) = self.cursor.take() else {
				return Poll::Ready(None);
			};

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
