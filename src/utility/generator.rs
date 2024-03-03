use super::{AsTraitEq, TraitEq};
use crate::kdl_ext::NodeContext;
use kdlize::{AsKdl, NodeId};
use std::{fmt::Debug, sync::Arc};

pub trait Generator: Debug + TraitEq + AsTraitEq<dyn TraitEq> + NodeId + AsKdl {}

pub type ArcGenerator = Arc<dyn Generator + 'static + Send + Sync>;
#[derive(Clone)]
pub struct GenericGenerator(ArcGenerator);

kdlize::impl_kdl_node!(GenericGenerator, "generator");

impl<M> From<M> for GenericGenerator
where
	M: Generator + 'static + Send + Sync,
{
	fn from(value: M) -> Self {
		Self(Arc::new(value))
	}
}

impl GenericGenerator {
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

impl PartialEq for GenericGenerator {
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl std::ops::Deref for GenericGenerator {
	type Target = Arc<dyn Generator + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl std::ops::DerefMut for GenericGenerator {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl std::fmt::Debug for GenericGenerator {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl AsKdl for GenericGenerator {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		crate::kdl_ext::NodeBuilder::default()
			.with_entry(self.0.get_id())
			.with_extension(self.0.as_kdl())
	}
}

impl kdlize::FromKdl<NodeContext> for GenericGenerator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.next_str_req()?;
		let node_reg = node.context().node_reg().clone();
		let factory = node_reg.get_generator_factory(id)?;
		factory.from_kdl(node)
	}
}
