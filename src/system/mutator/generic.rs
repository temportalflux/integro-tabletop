use super::{ArcMutator, Mutator};
use crate::kdl_ext::NodeContext;
use kdlize::{AsKdl, FromKdl};
use std::sync::Arc;

#[derive(Clone)]
pub struct Generic<T>(ArcMutator<T>);

impl<M, T> From<M> for Generic<T>
where
	M: Mutator<Target = T> + 'static + Send + Sync,
{
	fn from(value: M) -> Self {
		Self(Arc::new(value))
	}
}

impl<T> Generic<T> {
	pub fn new(value: ArcMutator<T>) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcMutator<T> {
		self.0
	}
}

impl<T> PartialEq for Generic<T>
where
	T: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl<T> std::ops::Deref for Generic<T> {
	type Target = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T> std::ops::DerefMut for Generic<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T> std::fmt::Debug for Generic<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<T> AsKdl for Generic<T> {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		crate::kdl_ext::NodeBuilder::default()
			.with_entry(self.0.get_id())
			.with_extension(self.0.as_kdl())
	}
}

impl<T> FromKdl<NodeContext> for Generic<T>
where
	T: 'static,
{
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.next_str_req()?;
		let node_reg = node.context().node_reg().clone();
		let factory = node_reg.get_mutator_factory(id)?;
		factory.from_kdl::<T>(node)
	}
}
