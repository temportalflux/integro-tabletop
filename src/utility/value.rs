use super::{Evaluator, RcEvaluator};
use std::rc::Rc;

#[derive(Clone)]
pub enum Value<C, V> {
	Fixed(V),
	Evaluated(RcEvaluator<C, V>),
}

impl<C, V> Default for Value<C, V>
where
	V: Default,
{
	fn default() -> Self {
		Self::Fixed(V::default())
	}
}

impl<C, V> PartialEq for Value<C, V>
where
	V: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Fixed(a), Self::Fixed(b)) => a == b,
			(Self::Evaluated(a), Self::Evaluated(b)) => Rc::ptr_eq(a, b),
			_ => false,
		}
	}
}

impl<C, V> std::fmt::Debug for Value<C, V>
where
	V: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Fixed(value) => write!(f, "Value::Fixed({value:?})"),
			Self::Evaluated(_eval) => write!(f, "Value::Evaluated(?)"),
		}
	}
}

impl<C, V> Evaluator for Value<C, V>
where
	V: Clone,
{
	type Context = C;
	type Item = V;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::Fixed(value) => value.clone(),
			Self::Evaluated(evaluator) => evaluator.evaluate(state),
		}
	}
}
