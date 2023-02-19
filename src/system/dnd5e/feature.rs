use super::{
	character::DerivedBuilder,
	condition::BoxedCondition,
	criteria::BoxedCriteria,
	mutator::{self, BoxedMutator},
	Action, Value,
};
use std::rc::Rc;

#[derive(Default, Clone, PartialEq)]
pub struct Feature {
	pub name: String,
	pub description: String,
	pub action: Option<Action>,
	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,
	pub limited_uses: Option<LimitedUses>,
}

impl mutator::Container for Feature {
	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
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
pub struct BoxedFeature(Rc<Feature>);
impl From<Feature> for BoxedFeature {
	fn from(feature: Feature) -> Self {
		Self(Rc::new(feature))
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
