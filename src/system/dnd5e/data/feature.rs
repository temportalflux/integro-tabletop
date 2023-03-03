use super::{action::ActivationKind, character::Character, condition::BoxedCondition};
use crate::{
	system::dnd5e::{BoxedCriteria, BoxedMutator, Value},
	utility::MutatorGroup,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default, Clone, PartialEq)]
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

#[derive(Default, Clone, PartialEq)]
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
