use std::str::FromStr;

use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::Rest, FromKDL, Value},
	},
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct LimitedUses {
	/// The number of uses the feature has until it resets.
	/// TODO: Use a ScalingUses instead of Value, which always scale in relation to some evaluator (in most cases, get_level)
	pub max_uses: Value<Option<usize>>,
	/// Consumed uses resets when the user takes at least this rest
	/// (a reset on a short rest will also reset on long rest).
	pub reset_on: Option<Rest>,
}

impl FromKDL for LimitedUses {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let max_uses = {
			// Temporary code, until I can implement scaling uses
			let node = node.query_req("max_uses")?;
			let max_uses = node.get_i64(0)? as usize;
			Value::Fixed(Some(max_uses))
		};
		let reset_on = match node.query_str_opt("reset_on", 0)? {
			None => None,
			Some(str) => Some(Rest::from_str(str)?),
		};
		Ok(Self { max_uses, reset_on })
	}
}
