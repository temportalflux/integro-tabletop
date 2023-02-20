use super::{character::State, evaluator::BoxedEvaluator};
use std::rc::Rc;

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

impl<T> Value<T> {
	pub fn evaluate(&self, state: &State) -> T
	where
		T: Clone,
	{
		match self {
			Self::Fixed(value) => value.clone(),
			Self::Evaluated(evaluator) => evaluator.evaluate(state),
		}
	}
}
