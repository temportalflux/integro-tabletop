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
	pub systems: Vec<String>,
	pub tree_id: String,
}
impl RepositoryMetadata {
	pub fn module_id(&self) -> crate::system::core::ModuleId {
		crate::system::core::ModuleId::Github {
			user_org: self.owner.clone(),
			repository: self.name.clone(),
		}
	}
}
