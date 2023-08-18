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
use std::{collections::HashMap, path::PathBuf};

/// Holds the list of all bundles added via mutators, and fetched from the object provider.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AdditionalBundleCache {
	pending: Vec<AdditionalBundleData>,

	applied_bundles: Vec<AdditionalBundleData>,
	object_cache: HashMap<SourceId, Bundle>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdditionalBundleData {
	pub ids: Vec<SourceId>,
	pub source: PathBuf,
	pub propagate_source_as_parent_feature: bool,
}

impl AdditionalBundleCache {
	/// Inserts new pending bundles into the cache.
	pub fn insert(&mut self, bundle_data: AdditionalBundleData) {
		self.pending.push(bundle_data);
	}

	pub fn has_pending_objects(&self) -> bool {
		!self.pending.is_empty()
	}

	pub async fn update_objects(&mut self, provider: &ObjectCacheProvider) -> anyhow::Result<()> {
		for AdditionalBundleData { ids, .. } in &self.pending {
			for bundle_id in ids {
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
		}
		Ok(())
	}

	pub fn apply_mutators(&mut self, target: &mut Character) {
		let pending = self.pending.drain(..).collect::<Vec<_>>();
		for bundle_data in pending {
			for bundle_id in &bundle_data.ids {
				let bundle = self
					.object_cache
					.get_mut(&bundle_id)
					.expect("Objects must be fetched by `update_objects` before being applied");

				// this will overwrite the data_path for the cached bundle every time, but thats fine.
				bundle.set_data_path(&bundle_data.source);
				// ensure that the bundle, if configured to show as a feature, has the proper parent
				if let Some(feature_config) = &mut bundle.feature_config {
					if bundle_data.propagate_source_as_parent_feature {
						feature_config.parent_path = Some(bundle_data.source.to_owned());
					}
				}

				// apply the bundle to the character
				target.apply_from(bundle, &bundle_data.source);
			}
			self.applied_bundles.push(bundle_data);
		}
	}
}

impl std::ops::AddAssign for AdditionalBundleCache {
	fn add_assign(&mut self, mut rhs: Self) {
		self.pending.append(&mut rhs.pending);
	}
}
