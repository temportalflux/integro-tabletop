use super::super::{Error, Record};

pub trait TransactionExt {
	fn object_store_of<T: Record>(&self) -> Result<idb::ObjectStore, Error>;
}

impl TransactionExt for idb::Transaction {
	fn object_store_of<T: Record>(&self) -> Result<idb::ObjectStore, Error> {
		Ok(self.object_store(T::store_id())?)
	}
}
