#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
	#[error("{0}")]
	Internal(String),
	#[error("{0}")]
	Serialization(String),
}

impl From<idb::Error> for Error {
	fn from(value: idb::Error) -> Self {
		Self::Internal(value.to_string())
	}
}

impl From<serde_wasm_bindgen::Error> for Error {
	fn from(value: serde_wasm_bindgen::Error) -> Self {
		Self::Serialization(value.to_string())
	}
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum UpgradeError {
	#[error("{0}")]
	Internal(String),
	#[error(transparent)]
	MissingVersion(#[from] MissingVersion),
}
impl From<idb::Error> for UpgradeError {
	fn from(value: idb::Error) -> Self {
		Self::Internal(format!("{value:?}"))
	}
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Schema is missing version {0}.")]
pub struct MissingVersion(pub u32);
