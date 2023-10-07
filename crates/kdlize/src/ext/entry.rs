use crate::error::MissingEntryType;

pub trait EntryExt {
	/// Returns the type of the entry.
	/// If the entry does not have a type, None is returned.
	fn type_opt(&self) -> Option<&str>;
	/// Returns the type of the entry.
	/// If the entry does not have a type, an error is returned.
	fn type_req(&self) -> Result<&str, MissingEntryType>;
}

impl EntryExt for kdl::KdlEntry {
	fn type_opt(&self) -> Option<&str> {
		self.ty().map(|id| id.value())
	}

	fn type_req(&self) -> Result<&str, MissingEntryType> {
		Ok(self.type_opt().ok_or(MissingEntryType(self.clone()))?)
	}
}
