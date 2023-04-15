#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
	#[error("{0}")]
	DatabaseError(String),
	#[error(transparent)]
	MissingSchemaVersion(#[from] MissingVersion),
	#[error("{0}")]
	FailedToSerialize(String),
}

impl From<idb::Error> for Error {
	fn from(value: idb::Error) -> Self {
		Self::DatabaseError(value.to_string())
	}
}

impl From<serde_wasm_bindgen::Error> for Error {
	fn from(value: serde_wasm_bindgen::Error) -> Self {
		Self::FailedToSerialize(value.to_string())
	}
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("Schema is missing version {0}.")]
pub struct MissingVersion(pub u32);
