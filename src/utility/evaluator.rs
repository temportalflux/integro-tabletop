use super::{AsTraitEq, Dependencies, TraitEq};
use crate::kdl_ext::{AsKdl, KDLNode};
use downcast_rs::{impl_downcast, DowncastSync};
use std::{fmt::Debug, sync::Arc};

pub trait Evaluator:
	DowncastSync + Debug + TraitEq + AsTraitEq<dyn TraitEq> + KDLNode + AsKdl
{
	type Context;
	type Item;

	/// The mutators this evaluator depends on.
	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn description(&self) -> Option<String>;

	fn evaluate(&self, context: &Self::Context) -> Self::Item;
}
impl_downcast!(Evaluator assoc Context, Item);

pub type ArcEvaluator<C, V> = Arc<dyn Evaluator<Context = C, Item = V> + 'static + Send + Sync>;
#[derive(Clone)]
pub struct GenericEvaluator<C, V>(ArcEvaluator<C, V>);

impl<T, C, V> From<T> for GenericEvaluator<C, V>
where
	T: Evaluator<Context = C, Item = V> + 'static + Send + Sync,
{
	fn from(value: T) -> Self {
		Self(Arc::new(value))
	}
}

impl<C, V> GenericEvaluator<C, V> {
	pub fn new(value: ArcEvaluator<C, V>) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcEvaluator<C, V> {
		self.0
	}
}

impl<C, V> PartialEq for GenericEvaluator<C, V>
where
	C: 'static,
	V: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl<C, V> std::ops::Deref for GenericEvaluator<C, V> {
	type Target = ArcEvaluator<C, V>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<C, V> std::fmt::Debug for GenericEvaluator<C, V> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<C, V> crate::kdl_ext::FromKDL for GenericEvaluator<C, V>
where
	C: 'static,
	V: 'static,
{
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.next_str_req()?;
		let node_reg = node.node_reg().clone();
		let factory = node_reg.get_evaluator_factory(id)?;
		factory.from_kdl::<C, V>(node)
	}
}

impl<C, V> AsKdl for GenericEvaluator<C, V> {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		crate::kdl_ext::NodeBuilder::default()
			.with_entry(self.0.get_id())
			.with_extension(self.0.as_kdl())
	}
}
