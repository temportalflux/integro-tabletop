
pub mod auth;
pub mod bootstrap;
pub mod components;
pub mod data;
pub mod database;
pub mod kdl_ext;
pub mod logging;
pub mod page;
pub mod path_map;
pub mod storage;
pub mod system;
pub mod task;
pub mod theme;
pub mod utility;

#[derive(thiserror::Error, Debug)]
pub struct GeneralError(pub String);
impl std::fmt::Display for GeneralError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
