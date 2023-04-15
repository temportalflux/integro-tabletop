//! A client-only IndexedDB is used to cache module data, so the application does not need to fetch all the contents of every module every time the app is opened. The database can be fully or partially refreshed as needed. Content is stored as raw kdl text in database entries, which can be parsed on the fly as content is needed for display or usage.
//! As of 2023.04.15, it is unclear to what degree this will replace system structures like `DnD5e`. There may be some data (like conditions) which need to stay in memory for easy access, while others (like items and spells) only need to be loaded when browsing content and relevant entries are loaded because they are a part of the character.
//! Each entry in the database is stored generically. It has a system id (e.g. `dnd5e`), a category (e.g. `item`, `spell`, `class`, `background`, etc.), and the kdl data associated with it. In the future, this could also include a `json` field for quickly converting between database and in-memory struct if kdl parsing proves to be too slow for on-the-fly usage.
use serde::Serialize;
use wasm_bindgen::JsValue;

mod client;
pub use client::*;
mod db;
pub use db::*;
mod error;
pub use error::*;
mod ext;
pub use ext::*;
mod index;
pub use index::*;
mod cursor;
pub use cursor::*;

pub trait Schema {
	fn latest() -> u32;
	fn apply(&self, database: &idb::Database) -> Result<(), Error>;
}

pub trait Record: Serialize {
	fn as_value(&self) -> Result<JsValue, serde_wasm_bindgen::Error> {
		Ok(self.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
	}
}

pub fn query<T: Into<JsValue>, const N: usize>(items: [T; N]) -> Result<idb::Query, Error> {
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
