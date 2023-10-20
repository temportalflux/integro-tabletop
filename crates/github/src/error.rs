#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
	#[error(transparent)]
	Request(std::sync::Arc<reqwest::Error>),
	#[error(transparent)]
	Deserialization(std::sync::Arc<serde_json::Error>),
	#[error("{0:?}")]
	InvalidResponse(std::sync::Arc<String>),
}
impl From<reqwest::Error> for Error {
	fn from(value: reqwest::Error) -> Self {
		Self::Request(std::sync::Arc::new(value))
	}
}
impl From<serde_json::Error> for Error {
	fn from(value: serde_json::Error) -> Self {
		Self::Deserialization(std::sync::Arc::new(value))
	}
}
