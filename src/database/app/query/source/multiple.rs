use crate::database::{app::QuerySource, Database};
use database::Record;

/// Queries the database for a specific record given a list of keys and a record type.
pub struct QueryMultiple<RecordType> {
	keys: Vec<String>,
	marker: std::marker::PhantomData<RecordType>,
}

impl<'iter, KeyIter, Key, RecordType> From<KeyIter> for QueryMultiple<RecordType>
where
	KeyIter: Iterator<Item = &'iter Key>,
	Key: ToString + 'iter,
	RecordType: Record + serde::de::DeserializeOwned,
{
	fn from(iter: KeyIter) -> Self {
		Self {
			keys: iter.map(ToString::to_string).collect(),
			marker: Default::default(),
		}
	}
}

impl<RecordType> QuerySource for QueryMultiple<RecordType>
where
	RecordType: Record + serde::de::DeserializeOwned,
{
	type Output = Result<Vec<RecordType>, database::Error>;

	async fn execute(self, database: &Database) -> Self::Output {
		let mut found = Vec::with_capacity(self.keys.len());
		for key in self.keys {
			if let Some(value) = database.get::<RecordType>(key).await? {
				found.push(value);
			}
		}
		Ok(found)
	}
}
