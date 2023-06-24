use crate::{
	system::{
		core::SourceId,
		dnd5e::data::{
			character::{Character, ObjectCacheProvider},
			Bundle,
		},
	},
	utility::MutatorGroup,
};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

/// Holds the list of all bundles added via mutators, and fetched from the object provider.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AdditionalBundleCache {
	pending: Vec<(SourceId, PathBuf)>,

	applied_bundles: Vec<(SourceId, PathBuf)>,
	object_cache: HashMap<SourceId, Bundle>,
}

impl AdditionalBundleCache {
	/// Inserts new pending bundles into the cache.
	pub fn insert(&mut self, bundle_ids: Vec<SourceId>, parent_path: &Path) {
		self.pending.reserve(bundle_ids.len());
		for id in bundle_ids {
			self.pending.push((id, parent_path.to_owned()));
		}
	}

	pub fn has_pending_objects(&self) -> bool {
		!self.pending.is_empty()
	}

	pub async fn update_objects(&mut self, provider: &ObjectCacheProvider) -> anyhow::Result<()> {
		for (bundle_id, _) in &self.pending {
			if !self.object_cache.contains_key(bundle_id) {
				let bundle = provider
					.database
					.get_typed_entry::<Bundle>(bundle_id.clone(), provider.system_depot.clone())
					.await?;
				let Some(bundle) = bundle else {
					log::error!(target: "bundle", "Failed to find bundle {:?}, no such entry in database.", bundle_id.to_string());
					continue;
				};
				self.object_cache.insert(bundle_id.clone(), bundle);
			}
		}
		Ok(())
	}

	pub fn apply_mutators(&mut self, target: &mut Character) {
		let pending = self.pending.drain(..).collect::<Vec<_>>();
		for (bundle_id, source) in pending {
			let bundle = self
				.object_cache
				.get_mut(&bundle_id)
				.expect("Objects must be fetched by `update_objects` before being applied");
			// this will overwrite the data_path for the cached bundle every time, but thats fine.
			bundle.set_data_path(&source);
			target.apply_from(bundle, &source);
			self.applied_bundles.push((bundle_id, source));
		}
	}
}

impl std::ops::AddAssign for AdditionalBundleCache {
	fn add_assign(&mut self, mut rhs: Self) {
		self.pending.append(&mut rhs.pending);
	}
}
