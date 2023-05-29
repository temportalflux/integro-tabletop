mod client;
pub use client::*;
pub mod queries;
mod query;
pub use query::*;

pub static MODULE_TOPIC: &str = "integro-tabletop-module";

#[derive(Clone, Debug)]
pub struct RepositoryMetadata {
	pub owner: String,
	pub name: String,
	pub is_private: bool,
	pub version: String,
}
