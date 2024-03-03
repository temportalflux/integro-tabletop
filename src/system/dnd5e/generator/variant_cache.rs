use crate::{database::Entry, system::core::SourceId};
use multimap::MultiMap;
use std::collections::HashMap;

/// A cache of entry records which were previously created by any generator.
/// Used to compare new entries with the same id to determine if running generators
/// has resulted in any meaninful changes to the generated entries (aka variants).
#[derive(Default)]
pub struct VariantCache {
	entries: HashMap<SourceId, Entry>,
	variants_by_generator: MultiMap<SourceId, SourceId>,
}

impl VariantCache {
	pub fn insert(&mut self, entry: Entry) {
		let Some(generator_id) = entry.generator_id() else {
			return;
		};
		let id = entry.source_id(false);
		self.variants_by_generator.insert(generator_id, id.clone());
		self.entries.insert(id, entry);
	}
}
