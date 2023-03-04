use super::{Evaluator, GenericEvaluator};
use std::{collections::HashSet, fmt::Debug, ops::Deref, sync::Arc};

#[derive(Clone)]
pub enum Value<C, V> {
	Fixed(V),
	Evaluated(GenericEvaluator<C, V>),
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
			(Self::Evaluated(a), Self::Evaluated(b)) => Arc::ptr_eq(a, b),
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
			Self::Evaluated(eval) => write!(f, "Value::Evaluated({eval:?})"),
		}
	}
}

impl<C, V> Evaluator for Value<C, V>
where
	C: 'static + Send + Sync,
	V: 'static + Clone + Send + Sync + Debug,
{
	type Context = C;
	type Item = V;

	fn dependencies(&self) -> Dependencies {
		match self {
			Self::Fixed(_) => Dependencies::default(),
			Self::Evaluated(evaluator) => evaluator.dependencies(),
		}
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::Fixed(value) => value.clone(),
			Self::Evaluated(evaluator) => evaluator.evaluate(state),
		}
	}
}

#[derive(Clone, PartialEq, Default)]
pub struct Dependencies(Option<HashSet<&'static str>>);
impl<const N: usize> From<[&'static str; N]> for Dependencies {
	fn from(values: [&'static str; N]) -> Self {
		Self(Some(HashSet::from(values)))
	}
}
impl std::fmt::Debug for Dependencies {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.0 {
			None => write!(f, "{{}}"),
			Some(deps) => write!(f, "{:?}", deps),
		}
	}
}
impl Dependencies {
	pub fn join(self, other: Self) -> Self {
		match (self.0, other.0) {
			(None, None) => Self(None),
			(None, Some(deps)) | (Some(deps), None) => Self(Some(deps)),
			(Some(a), Some(b)) => Self(Some(a.union(&b).cloned().collect())),
		}
	}
}
impl Deref for Dependencies {
	type Target = Option<HashSet<&'static str>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
