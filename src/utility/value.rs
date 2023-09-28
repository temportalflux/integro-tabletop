use super::GenericEvaluator;
use crate::{
	kdl_ext::{AsKdl, EntryExt, NodeBuilder, ValueExt},
	system::dnd5e::data::character::Character,
};
use anyhow::Context;
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

impl<C, V> Value<C, V>
where
	C: 'static + Send + Sync,
	V: 'static + Clone + Send + Sync + Debug + PartialEq + ToString,
{
	pub fn dependencies(&self) -> Dependencies {
		match self {
			Self::Fixed(_) => Dependencies::default(),
			Self::Evaluated(evaluator) => evaluator.dependencies(),
		}
	}

	pub fn description(&self) -> Option<String> {
		match self {
			Value::Fixed(value) => Some(value.to_string()),
			Value::Evaluated(eval) => eval.description(),
		}
	}

	pub fn evaluate(&self, state: &C) -> V {
		match self {
			Self::Fixed(value) => value.clone(),
			Self::Evaluated(evaluator) => evaluator.evaluate(state),
		}
	}
}

pub trait FromKdlValue {
	fn parse(value: &kdl::KdlValue) -> anyhow::Result<Self>
	where
		Self: Sized;
}
impl FromKdlValue for bool {
	fn parse(value: &kdl::KdlValue) -> anyhow::Result<Self>
	where
		Self: Sized,
	{
		Ok(value.as_bool_req()?)
	}
}
impl FromKdlValue for i32 {
	fn parse(value: &kdl::KdlValue) -> anyhow::Result<Self>
	where
		Self: Sized,
	{
		Ok(value.as_i64_req()? as i32)
	}
}
impl FromKdlValue for String {
	fn parse(value: &kdl::KdlValue) -> anyhow::Result<Self>
	where
		Self: Sized,
	{
		Ok(value.as_str_req()?.to_owned())
	}
}

// TODO: Test Value::from_kdl/as_kdl
impl<V> crate::kdl_ext::FromKDL for Value<Character, V>
where
	V: 'static + FromKdlValue,
{
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		match entry.type_opt() {
			Some("Evaluator") => {
				let eval_id = entry
					.as_str_req()
					.context("Evaluator values must be a string containing the evaluator id")?;
				let node_reg = node.node_reg().clone();
				let factory = node_reg.get_evaluator_factory(eval_id)?;
				Ok(Self::Evaluated(factory.from_kdl::<Character, V>(node)?))
			}
			_ => Ok(Self::Fixed(V::parse(entry.value())?)),
		}
	}
}
impl<V: 'static + AsKdl> AsKdl for Value<Character, V> {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Fixed(value) => value.as_kdl(),
			Self::Evaluated(eval) => {
				let mut node = eval.as_kdl();
				node.set_first_entry_ty("Evaluator");
				node
			}
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

impl std::ops::AddAssign for Dependencies {
	fn add_assign(&mut self, rhs: Self) {
		let mut tmp = Self::default();
		std::mem::swap(self, &mut tmp);
		*self = tmp.join(rhs);
	}
}
