use crate::{
	kdl_ext::{NodeContext, NodeReader},
	utility::BoxAny,
};
use kdlize::FromKdl;
use std::sync::Arc;

use super::{ArcGenerator, Generator, Generic};

pub struct Factory {
	fn_from_kdl: Box<dyn Fn(&mut NodeReader<'_>) -> anyhow::Result<BoxAny> + 'static + Send + Sync>,
}

impl Factory {
	pub fn new<M>() -> Self
	where
		M: Generator + FromKdl<NodeContext> + 'static + Send + Sync,
		anyhow::Error: From<M::Error>,
	{
		Self {
			fn_from_kdl: Box::new(|node| {
				let arc_eval: ArcGenerator = Arc::new(M::from_kdl(node)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<'doc>(&self, node: &mut NodeReader<'doc>) -> anyhow::Result<Generic> {
		let any = (self.fn_from_kdl)(node)?;
		let eval = any
			.downcast::<ArcGenerator>()
			.expect("failed to unpack boxed arc-generator");
		Ok(Generic::new(*eval))
	}
}
