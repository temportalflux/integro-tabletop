pub trait Schema {
	fn latest() -> u32;
	fn apply(&self, database: &idb::Database, transaction: Option<&idb::Transaction>) -> Result<(), idb::Error>;
}
