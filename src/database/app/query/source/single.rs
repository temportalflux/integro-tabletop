use crate::database::{app::QuerySource, Database};
use database::Record;

/// Queries the database for a specific record given its key and type.
pub struct QuerySingle<RecordType> {
	key: String,
	marker: std::marker::PhantomData<RecordType>,
}

impl<Key, RecordType> From<Key> for QuerySingle<RecordType>
where
	Key: ToString,
	RecordType: Record + serde::de::DeserializeOwned,
{
	fn from(value: Key) -> Self {
		Self {
			key: value.to_string(),
			marker: Default::default(),
		}
	}
}

impl<RecordType> QuerySource for QuerySingle<RecordType>
where
	RecordType: Record + serde::de::DeserializeOwned,
{
	type Output = Result<Option<RecordType>, database::Error>;

	#[must_use]
	async fn execute(self, database: &Database) -> Self::Output {
		database.get::<RecordType>(self.key).await
	}
}
