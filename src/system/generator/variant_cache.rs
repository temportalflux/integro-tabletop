use crate::{database::Entry, system::SourceId};
use std::collections::HashMap;

/// A cache of entry records which were previously created by any generator.
/// Used to compare new entries with the same id to determine if running generators
/// has resulted in any meaninful changes to the generated entries (aka variants).
#[derive(Default)]
pub struct VariantCache {
	// variants which are present in the database and for which no updates have yet been inserted.
	stale: HashMap<SourceId, Entry>,
	// variants which were present in the database, and an update was found which was equal to the orignial.
	unchanged: HashMap<SourceId, Entry>,
	// variants which were present in the database, and an update was found which has changed compared to the original.
	updated: HashMap<SourceId, Entry>,
	// variants which were not previously present in the database.
	new: HashMap<SourceId, Entry>,
}

impl VariantCache {
	pub fn insert_original(&mut self, entry: Entry) {
		let id = entry.source_id(false);
		self.stale.insert(id, entry);
	}

	pub fn insert_update(&mut self, entry: Entry) {
		let id = entry.source_id(false);
		// Remove from being stale. If no old entry exists, the the one provided is new.
		let Some(prev) = self.stale.remove(&id) else {
			self.new.insert(id, entry);
			return;
		};
		// Entries with the same serialized kdl content are considered identical,
		// since their metadata & category are both derived from kdl content,
		// and the module & system are both derived from the source id,
		// and the version, file_id, generator data, etc are all irrelevant for comparison.
		if entry.kdl == prev.kdl {
			// if unchanged, just preserve previous.
			self.unchanged.insert(id, prev);
			return;
		}
		// If a previous entry existed and the new entry has different serialized content,
		// then it has changed and we should discard prev in favor of new.
		self.updated.insert(id, entry);
	}

	pub fn drain_new(&mut self) -> impl Iterator<Item = Entry> + '_ {
		self.new.drain().map(|(_id, entry)| entry)
	}

	pub fn drain_updated(&mut self) -> impl Iterator<Item = Entry> + '_ {
		self.updated.drain().map(|(_id, entry)| entry)
	}

	pub fn drain_stale(&mut self) -> impl Iterator<Item = Entry> + '_ {
		self.stale.drain().map(|(_id, entry)| entry)
	}
}
