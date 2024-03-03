use super::{ArcEvaluator, Evaluator, Generic};
use crate::{
	kdl_ext::{NodeContext, NodeReader},
	utility::{BoxAny, IncompatibleTypes},
};
use kdlize::FromKdl;
use std::{any::TypeId, sync::Arc};

pub struct Factory {
	type_name: &'static str,
	/// Information about the expected output type of the evaluator.
	/// Used to ensure the expected output type of `from_kdl` matches
	/// that of the registered evaluator, otherwise Any downcast will implode.
	item_type_info: (TypeId, &'static str),
	ctx_type_info: (TypeId, &'static str),
	fn_from_kdl: Box<dyn Fn(&mut NodeReader<'_>) -> anyhow::Result<BoxAny> + 'static + Send + Sync>,
}

impl Factory {
	pub fn new<E>() -> Self
	where
		E: Evaluator + FromKdl<NodeContext> + 'static + Send + Sync,
		anyhow::Error: From<E::Error>,
	{
		Self {
			type_name: std::any::type_name::<E>(),
			ctx_type_info: (TypeId::of::<E::Context>(), std::any::type_name::<E::Context>()),
			item_type_info: (TypeId::of::<E::Item>(), std::any::type_name::<E::Item>()),
			fn_from_kdl: Box::new(|node| {
				let arc_eval: ArcEvaluator<E::Context, E::Item> = Arc::new(E::from_kdl(node)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<'doc, C, T>(&self, node: &mut NodeReader<'doc>) -> anyhow::Result<Generic<C, T>>
	where
		C: 'static,
		T: 'static,
	{
		if TypeId::of::<C>() != self.ctx_type_info.0 {
			return Err(IncompatibleTypes(
				"context",
				self.type_name,
				self.ctx_type_info.1,
				std::any::type_name::<C>(),
			)
			.into());
		}

		if TypeId::of::<T>() != self.item_type_info.0 {
			return Err(IncompatibleTypes(
				"output",
				self.type_name,
				self.item_type_info.1,
				std::any::type_name::<T>(),
			)
			.into());
		}

		let any = (self.fn_from_kdl)(node)?;
		let eval = any
			.downcast::<ArcEvaluator<C, T>>()
			.expect("failed to unpack boxed arc-evaluator");
		Ok(Generic::new(*eval))
	}
}
