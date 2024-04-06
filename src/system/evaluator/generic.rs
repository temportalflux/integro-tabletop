use super::{ArcEvaluator, Evaluator};
use crate::kdl_ext::{NodeBuilder, NodeContext, NodeReader};
use kdlize::{AsKdl, FromKdl};
use std::sync::Arc;

#[derive(Clone)]
pub struct Generic<C, V>(ArcEvaluator<C, V>);

impl<T, C, V> From<T> for Generic<C, V>
where
	T: Evaluator<Context = C, Item = V> + 'static + Send + Sync,
{
	fn from(value: T) -> Self {
		Self(Arc::new(value))
	}
}

impl<C, V> Generic<C, V> {
	pub fn new(value: ArcEvaluator<C, V>) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcEvaluator<C, V> {
		self.0
	}
}

impl<C, V> PartialEq for Generic<C, V>
where
	C: 'static,
	V: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl<C, V> std::ops::Deref for Generic<C, V> {
	type Target = ArcEvaluator<C, V>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<C, V> std::fmt::Debug for Generic<C, V> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<C, V> FromKdl<NodeContext> for Generic<C, V>
where
	C: 'static,
	V: 'static,
{
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.next_str_req()?;
		let node_reg = node.context().node_reg().clone();
		let factory = node_reg.get_evaluator_factory(id)?;
		factory.from_kdl::<C, V>(node)
	}
}

impl<C, V> AsKdl for Generic<C, V> {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.0.get_id());
		node += self.0.as_kdl();
		node
	}
}
