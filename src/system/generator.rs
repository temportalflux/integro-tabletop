use crate::kdl_ext::NodeContext;
use crate::system::{Block, SourceId};
use crate::utility::{AsTraitEq, PinFuture, TraitEq};
use database::Transaction;
use kdlize::{AsKdl, NodeId};
use std::{fmt::Debug, sync::Arc};

mod factory;
pub use factory::*;
mod generic;
pub use generic::*;

#[derive(Default)]
pub struct SystemObjectList {}

impl SystemObjectList {
	pub fn insert<T>(&mut self, object: T)
	where
		T: Block + 'static + Send + Sync,
	{
	}
}

pub trait Generator: Debug + TraitEq + AsTraitEq<dyn TraitEq> + NodeId + AsKdl {
	fn source_id(&self) -> &SourceId;
	fn execute(&self, context: &NodeContext, transaction: &Transaction) -> PinFuture<anyhow::Result<SystemObjectList>>;
}

pub type ArcGenerator = Arc<dyn Generator + 'static + Send + Sync>;
