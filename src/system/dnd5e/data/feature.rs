use super::{action::ActivationKind, character::Character, condition::BoxedCondition};
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt},
	system::dnd5e::{BoxedCriteria, BoxedMutator, DnD5e, FromKDL, Value},
	utility::MutatorGroup,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Feature {
	pub name: String,
	pub description: String,

	// TODO: Vec of Actions which are added when applied. Each action has the activation and a description, as already supported by Weapons.
	// This is in addition to the existing action + limited uses (which allows the feature to display in the actions panel).
	pub action: Option<ActivationKind>,
	pub limited_uses: Option<LimitedUses>,

	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,

	pub missing_selection_text: Option<(String, HashMap<String, String>)>,
}

impl Feature {
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

	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		if let Some(criteria) = &self.criteria {
			// TODO: Somehow save the error text for display in feature UI
			if stats.evaluate(criteria).is_err() {
				return;
			}
		}
		for mutator in &self.mutators {
			stats.apply(mutator);
		}
	}
}

impl FromKDL for Feature {
	type System = DnD5e;

	fn from_kdl(node: &kdl::KdlNode, system: &Self::System) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.unwrap_or_default()
			.to_owned();
		// Specifies if this feature can appear twice.
		// If true, any other features with the same name are ignored/discarded.
		// TODO: Unimplemented
		let _is_unique = node.get_bool_opt("unique")?.unwrap_or_default();

		let criteria = match node.query("criteria")? {
			None => None,
			Some(entry_node) => {
				let id = entry_node.get_str(0)?;
				let factory = system.get_evaluator_factory(id)?;
				Some(factory.from_kdl::<Result<(), String>>(entry_node, system)?)
			}
		};

		// TODO: These
		let action = Default::default();
		let limited_uses = Default::default();

		let mut mutators = Vec::new();
		if let Some(children) = node.children() {
			for entry_node in children.query_all("mutator")? {
				let id = entry_node.get_str(0)?;
				let factory = system.get_mutator_factory(id)?;
				mutators.push(factory.from_kdl(entry_node, system)?);
			}
		}

		Ok(Self {
			name,
			description,
			mutators,
			action,
			limited_uses,
			criteria,
			// Generated data
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

#[derive(Default, Clone, PartialEq, Debug)]
pub struct LimitedUses {
	/// The number of uses the feature has until it resets.
	pub max_uses: Value<Option<usize>>,
	/// Consumed uses resets when the user takes at least this rest
	/// (a reset on a short rest will also reset on long rest).
	pub reset_on: Option<Rest>,
	pub apply_conditions: Vec<BoxedCondition>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Rest {
	Short,
	Long,
}
