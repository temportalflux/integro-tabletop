#[derive(thiserror::Error, Debug)]
pub enum Error {
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
