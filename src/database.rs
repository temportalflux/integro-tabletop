//! A client-only IndexedDB is used to cache module data, so the application does not need to fetch all the contents of every module every time the app is opened. The database can be fully or partially refreshed as needed. Content is stored as raw kdl text in database entries, which can be parsed on the fly as content is needed for display or usage.
//! Each entry in the database is stored generically. It has a system id (e.g. `dnd5e`), a category (e.g. `item`, `spell`, `class`, `background`, etc.), and the kdl data associated with it. In the future, this could also include a `json` field for quickly converting between database and in-memory struct if kdl parsing proves to be too slow for on-the-fly usage.
mod app;
pub use app::*;
