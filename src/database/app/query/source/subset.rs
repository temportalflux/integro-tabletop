use crate::database::{app::QuerySource, Database};
use database::{Cursor, IndexType, Record};

// Queries the database for all records in a given index (i.e. segment or partition of the database).
pub struct QuerySubset<Index>(Option<Index>);

impl<Index> From<Option<Index>> for QuerySubset<Index>
where
	Index: IndexType,
	Index::Record: Record + serde::de::DeserializeOwned,
{
	fn from(value: Option<Index>) -> Self {
		Self(value)
	}
}

impl<Index> QuerySource for QuerySubset<Index>
where
	Index: IndexType,
	Index::Record: Record + serde::de::DeserializeOwned,
{
	type Output = Result<Cursor<Index::Record>, database::Error>;

	async fn execute(self, database: &Database) -> Self::Output {
		use database::{ObjectStoreExt, TransactionExt};
		let transaction = database.read_only::<Index::Record>()?;
		let object_store = transaction.object_store_of::<Index::Record>()?;
		let index_table = object_store.index_of::<Index>()?;
		let cursor = index_table.open_cursor(self.0.as_ref()).await?;
		Ok(cursor)
	}
}
