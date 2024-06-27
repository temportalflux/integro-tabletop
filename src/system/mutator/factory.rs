use super::{ArcMutator, Generic, Mutator};
use crate::{
	kdl_ext::{NodeContext, NodeReader},
	utility::{BoxAny, IncompatibleTypes},
};
use kdlize::FromKdl;
use std::{any::TypeId, sync::Arc};

pub struct Factory {
	type_name: &'static str,
	target_type_info: (TypeId, &'static str),
	fn_from_kdl: Box<dyn Fn(&mut NodeReader<'_>) -> anyhow::Result<BoxAny> + 'static + Send + Sync>,
}

impl Factory {
	pub fn new<M>() -> Self
	where
		M: Mutator + FromKdl<NodeContext> + 'static + Send + Sync,
		anyhow::Error: From<M::Error>,
	{
		Self {
			type_name: std::any::type_name::<M>(),
			target_type_info: (TypeId::of::<M::Target>(), std::any::type_name::<M::Target>()),
			fn_from_kdl: Box::new(|node| {
				let arc_eval: ArcMutator<M::Target> = Arc::new(M::from_kdl(node)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<'doc, T>(&self, node: &mut NodeReader<'doc>) -> anyhow::Result<Generic<T>>
	where
		T: 'static,
	{
		if TypeId::of::<T>() != self.target_type_info.0 {
			return Err(IncompatibleTypes(
				"target",
				self.type_name,
				self.target_type_info.1,
				std::any::type_name::<T>(),
			)
			.into());
		}
		let any = (self.fn_from_kdl)(node)?;
		let eval = any.downcast::<ArcMutator<T>>().expect("failed to unpack boxed arc-evaluator");
		Ok(Generic::new(*eval))
	}
}
