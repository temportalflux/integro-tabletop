use super::{Evaluator, GenericEvaluator};
use crate::{kdl_ext::ValueIdx, GeneralError};
use std::{collections::HashSet, fmt::Debug, ops::Deref};

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
	C: 'static,
	V: 'static + PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Fixed(a), Self::Fixed(b)) => a == b,
			(Self::Evaluated(a), Self::Evaluated(b)) => a == b,
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

impl<C, V> super::TraitEq for Value<C, V>
where
	C: 'static,
	V: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn super::TraitEq) -> bool {
		super::downcast_trait_eq(self, other)
	}
}

impl<C, V> Evaluator for Value<C, V>
where
	C: 'static + Send + Sync,
	V: 'static + Clone + Send + Sync + Debug + PartialEq,
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

// TODO: Test Value::from_kdl
impl<V> Value<crate::system::dnd5e::data::character::Character, V>
where
	V: 'static,
{
	pub fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		system: &crate::system::dnd5e::DnD5e,
		map_value: impl Fn(&kdl::KdlValue) -> Option<V>,
	) -> anyhow::Result<Self> {
		let entry_idx = value_idx.next();
		let entry = node.entry(entry_idx).ok_or(GeneralError(format!(
			"Missing value at index {entry_idx} in node {node:?}"
		)))?;
		match entry.ty().map(|id| id.value()) {
			Some("Evaluator") => {
				let evaluator_name = entry.value().as_string().ok_or(GeneralError(format!(
					"Evaluator-typed values must be associated with a string, {entry:?} is not."
				)))?;
				let factory = system.get_evaluator_factory(evaluator_name)?;
				Ok(Self::Evaluated(
					factory.from_kdl::<V>(node, value_idx, system)?,
				))
			}
			_ => Ok(Self::Fixed(map_value(entry.value()).ok_or(
				GeneralError(format!(
					"Failed to parse value from {:?}, expected {:?}",
					entry.value(),
					std::any::type_name::<V>()
				)),
			)?)),
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
