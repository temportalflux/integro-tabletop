use super::{
	action::{Action, ActionSource},
	character::Character,
	description,
};
use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::{BoxedCriteria, BoxedMutator},
	utility::MutatorGroup,
};
use derivative::Derivative;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, RwLock},
};

// TODO: All actions are actually features. The action structure should not have a name or description.
// Instead, a feature can optionally have 1 action block, which must specify the activation type
// and all of the other propertyies about an action.
// To that end, a Feature is considered an action if it has an action block.
// This will re-unify the weird action vs feature breakdown. All non-action features are just passive buffs.
// This also aids in the unification of having categories for actions and features, and the display modal for both.
#[derive(Default, Clone, Debug, Derivative)]
#[derivative(PartialEq)]
pub struct Feature {
	pub name: String,
	pub description: description::Info,

	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,

	#[derivative(PartialEq = "ignore")]
	pub absolute_path: Arc<RwLock<PathBuf>>,
	#[derivative(PartialEq = "ignore")]
	pub missing_selection_text: Option<(String, HashMap<String, String>)>,
}

impl Feature {
	pub fn get_display_path(&self) -> PathBuf {
		self.absolute_path.read().unwrap().clone()
	}

	pub fn get_missing_selection_text_for(&self, key: &str) -> Option<&String> {
		let Some((default_text, specialized)) = &self.missing_selection_text else { return None; };
		if let Some(key_specific) = specialized.get(key) {
			return Some(key_specific);
		}
		Some(default_text)
	}
}

impl MutatorGroup for Feature {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		if let Some(criteria) = &self.criteria {
			// TODO: Somehow save the error text for display in feature UI
			if stats.evaluate(criteria).is_err() {
				return;
			}
		}
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
		*self.absolute_path.write().unwrap() = path_to_self;
	}
}

impl FromKDL for Feature {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = description::Info::from_kdl_all(node, ctx)?;

		// Specifies if this feature can appear twice.
		// If true, any other features with the same name are ignored/discarded.
		// TODO: Unimplemented
		let _is_unique = node.get_bool_opt("unique")?.unwrap_or_default();

		let criteria = match node.query("scope() > criteria")? {
			None => None,
			Some(entry_node) => {
				Some(ctx.parse_evaluator::<Character, Result<(), String>>(entry_node)?)
			}
		};

		for entry_node in node.query_all("scope() > action")? {
			log::warn!(
				target: "kdl",
				"Feature block does not currently support actions, \
				use the \"add_action\" mutator instead.\n{:?}",
				entry_node.to_string()
			);
		}

		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		Ok(Self {
			name,
			description,
			mutators,
			criteria,
			// Generated data
			absolute_path: Arc::new(RwLock::new(PathBuf::new())),
			missing_selection_text: None,
		})
	}
}

#[derive(Clone, PartialEq)]
pub struct BoxedFeature(Arc<Feature>);
impl From<Feature> for BoxedFeature {
	fn from(feature: Feature) -> Self {
		Self(Arc::new(feature))
	}
}
impl BoxedFeature {
	pub fn inner(&self) -> &Feature {
		&*self.0
	}
}
impl std::fmt::Debug for BoxedFeature {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}
impl MutatorGroup for BoxedFeature {
	type Target = <Feature as MutatorGroup>::Target;

	fn set_data_path(&self, parent: &std::path::Path) {
		self.0.set_data_path(parent);
	}

	fn apply_mutators(&self, target: &mut Self::Target, parent: &Path) {
		self.0.apply_mutators(target, parent);
	}
}
