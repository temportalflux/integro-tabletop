mod data;
pub use data::*;
mod id;
pub use id::IdPath;
mod object;
pub use object::*;
mod value;
pub use value::*;

#[derive(Clone, PartialEq, thiserror::Error, Debug)]
#[error("Invalid selector data path")]
pub struct InvalidDataPath;
