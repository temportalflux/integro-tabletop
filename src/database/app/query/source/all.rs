use crate::database::{app::QuerySource, Database};
use database::{Cursor, Record};

// Queries the database for all records of a particular type.
pub struct QueryAll<RecordType>(std::marker::PhantomData<RecordType>);

impl<RecordType> Default for QueryAll<RecordType> {
	fn default() -> Self {
		Self(Default::default())
	}
}

impl<RecordType> QuerySource for QueryAll<RecordType>
where
	RecordType: Record + serde::de::DeserializeOwned,
{
	type Output = Result<Cursor<RecordType>, database::Error>;

	async fn execute(self, database: &Database) -> Self::Output {
		use database::{ObjectStoreExt, TransactionExt};
		let transaction = database.read_only::<RecordType>()?;
		let object_store = transaction.object_store_of::<RecordType>()?;
		let cursor = object_store.cursor_all::<RecordType>().await?;
		Ok(cursor)
	}
}
