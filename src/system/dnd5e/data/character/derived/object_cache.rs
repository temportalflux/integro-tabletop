use crate::{
	kdl_ext::KDLNode,
	system::{
		core::SourceId,
		dnd5e::data::{
			character::{Character, ObjectCacheProvider},
			Bundle, Subclass,
		},
	},
	utility::MutatorGroup,
};
use std::{collections::HashMap, path::PathBuf};

/// Holds the list of all objects (mainly bundles) added via mutators, and fetched from the object provider.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct AdditionalObjectCache {
	// Objects which have not been applied to the character yet.
	// Entries may or may not be present in the `object_cache` yet.
	pending: Vec<AdditionalObjectData>,
	// Objects which have been applied to the character.
	// All of these entries must exist in the `object_cache`.
	applied_objects: Vec<AdditionalObjectData>,
	// The current cache of all known auxilary objects.
	object_cache: HashMap<SourceId, CachedObject>,
}

#[derive(Clone, PartialEq, Debug)]
enum CachedObject {
	Bundle(Bundle),
	Subclass(Subclass),
}

#[derive(Clone, PartialEq, Debug)]
pub struct AdditionalObjectData {
	pub ids: Vec<SourceId>,
	pub object_type_id: String,
	pub source: PathBuf,
	pub propagate_source_as_parent_feature: bool,
}

impl AdditionalObjectCache {
	/// Inserts new pending objects into the cache.
	pub fn insert(&mut self, object_data: AdditionalObjectData) {
		self.pending.push(object_data);
	}

	pub fn has_pending_objects(&self) -> bool {
		!self.pending.is_empty()
	}

	pub async fn update_objects(&mut self, provider: &ObjectCacheProvider) -> anyhow::Result<()> {
		// TODO: Because ObjectCacheProvider contains both the databaser and system depot,
		// we should be able to generically deserialize objects into system components,
		// and then store the generic data which is a MutatorGroup instead of the hard types.
		// We can re-serialize them in the same manner perhaps.
		for AdditionalObjectData {
			ids,
			object_type_id,
			..
		} in &self.pending
		{
			for object_id in ids {
				if self.object_cache.contains_key(object_id) {
					continue;
				}
				if object_type_id == Bundle::id() {
					let bundle = provider
						.database
						.get_typed_entry::<Bundle>(
							object_id.clone(),
							provider.system_depot.clone(),
							None,
						)
						.await?;
					let Some(bundle) = bundle else {
						log::error!(target: "object_cache", "Failed to find bundle {:?}, no such entry in database.", object_id.to_string());
						continue;
					};
					self.object_cache
						.insert(object_id.clone(), CachedObject::Bundle(bundle));
				} else if object_type_id == Subclass::id() {
					let subclass = provider
						.database
						.get_typed_entry::<Subclass>(
							object_id.clone(),
							provider.system_depot.clone(),
							None,
						)
						.await?;
					let Some(subclass) = subclass else {
						log::error!(target: "object_cache", "Failed to find subclass {:?}, no such entry in database.", object_id.to_string());
						continue;
					};
					self.object_cache
						.insert(object_id.clone(), CachedObject::Subclass(subclass));
				} else {
					log::error!(target: "object_cache", "AdditionalObjectCache does not currently support {object_type_id:?} objects.");
				}
			}
		}
		Ok(())
	}

	pub fn apply_mutators(&mut self, target: &mut Character) {
		let pending = self.pending.drain(..).collect::<Vec<_>>();
		for object_data in pending {
			for object_id in &object_data.ids {
				let cached_object = self
					.object_cache
					.get_mut(&object_id)
					.expect("Objects must be fetched by `update_objects` before being applied");

				match cached_object {
					CachedObject::Bundle(bundle) => {
						// this will overwrite the data_path for the cached bundle every time, but thats fine.
						bundle.set_data_path(&object_data.source);
						// ensure that the bundle, if configured to show as a feature, has the proper parent
						if let Some(feature_config) = &mut bundle.feature_config {
							if object_data.propagate_source_as_parent_feature {
								feature_config.parent_path = Some(object_data.source.to_owned());
							}
						}
						// apply the bundle to the character
						target.apply_from(bundle, &object_data.source);
					}
					CachedObject::Subclass(subclass) => {
						// this will overwrite the data_path for the cached subclass every time, but thats fine.
						subclass.set_data_path(&object_data.source);
						// apply the subclass to the character
						target.apply_from(subclass, &object_data.source);
					}
				}
			}
			self.applied_objects.push(object_data);
		}
	}
}

impl std::ops::AddAssign for AdditionalObjectCache {
	fn add_assign(&mut self, mut rhs: Self) {
		self.pending.append(&mut rhs.pending);
	}
}
