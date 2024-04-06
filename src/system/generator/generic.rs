use super::{ArcGenerator, Generator};
use crate::{kdl_ext::NodeContext, system::Block};
use kdlize::AsKdl;
use std::sync::Arc;

#[derive(Clone)]
pub struct Generic(ArcGenerator);

kdlize::impl_kdl_node!(Generic, "generator");

impl Block for Generic {
	fn to_metadata(self) -> serde_json::Value {
		// TODO: id (SourceId) and kind (<Generator as NodeId>::id) fields
		serde_json::json!(null)
	}
}

impl<M> From<M> for Generic
where
	M: Generator + 'static + Send + Sync,
{
	fn from(value: M) -> Self {
		Self(Arc::new(value))
	}
}

impl Generic {
	pub fn new(value: ArcGenerator) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcGenerator {
		self.0
	}

	/// Returns the `NodeId::id()` value of the inner generator.
	pub fn kind(&self) -> &'static str {
		self.0.get_id()
	}
}

impl PartialEq for Generic {
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl std::ops::Deref for Generic {
	type Target = Arc<dyn Generator + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl std::ops::DerefMut for Generic {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl std::fmt::Debug for Generic {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl kdlize::FromKdl<NodeContext> for Generic {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.next_str_req()?;
		let node_reg = node.context().node_reg().clone();
		let factory = node_reg.get_generator_factory(id)?;
		factory.from_kdl(node)
	}
}

impl AsKdl for Generic {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		let mut node = crate::kdl_ext::NodeBuilder::default();
		node.entry(self.0.get_id());
		node += self.0.as_kdl();
		node
	}
}
