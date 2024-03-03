use crate::utility::{AsTraitEq, Dependencies, TraitEq};
use downcast_rs::{impl_downcast, DowncastSync};
use kdlize::{AsKdl, NodeId};
use std::{fmt::Debug, sync::Arc};

mod factory;
pub use factory::*;
mod generic;
pub use generic::*;

pub type ArcEvaluator<C, V> = Arc<dyn Evaluator<Context = C, Item = V> + 'static + Send + Sync>;

pub trait Evaluator: DowncastSync + Debug + TraitEq + AsTraitEq<dyn TraitEq> + NodeId + AsKdl {
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
