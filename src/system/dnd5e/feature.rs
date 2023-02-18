use super::{
	character::DerivedBuilder,
	condition::BoxedCondition,
	criteria::BoxedCriteria,
	mutator::{self, BoxedMutator},
	Action,
};
use std::rc::Rc;

#[derive(Default, Clone, PartialEq)]
pub struct Feature {
	pub name: String,
	pub description: String,
	pub action: Option<Action>,
	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>, // TODO: Implement
	pub limited_uses: Option<LimitedUses>,
}

impl mutator::Container for Feature {
	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
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

#[derive(Clone)]
pub enum Value<T> {
	Fixed(T),
	Evaluated(BoxedEvaluator<T>),
}
impl<T> Default for Value<T>
where
	T: Default,
{
	fn default() -> Self {
		Self::Fixed(T::default())
	}
}
impl<T> PartialEq for Value<T>
where
	T: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Fixed(a), Self::Fixed(b)) => a == b,
			(Self::Evaluated(a), Self::Evaluated(b)) => Rc::ptr_eq(a, b),
			_ => false,
		}
	}
}

pub trait Evaluator {
	type Item;
}
#[derive(Clone)]
pub struct BoxedEvaluator<V>(std::rc::Rc<dyn Evaluator<Item = V> + 'static>);
impl<V> PartialEq for BoxedEvaluator<V> {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.0, &other.0)
	}
}
impl<V> std::ops::Deref for BoxedEvaluator<V> {
	type Target = std::rc::Rc<dyn Evaluator<Item = V> + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T, V> From<T> for BoxedEvaluator<V>
where
	T: Evaluator<Item = V> + 'static,
{
	fn from(value: T) -> Self {
		Self(std::rc::Rc::new(value))
	}
}
