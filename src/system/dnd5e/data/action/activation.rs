use crate::{
	kdl_ext::{FromKDL, NodeExt},
	GeneralError,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub enum ActivationKind {
	#[default]
	Action,
	Bonus,
	Reaction,
	Special,
	Minute(u32),
	Hour(u32),
}

impl FromKDL for ActivationKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			"Special" => Ok(Self::Special),
			"Minute" => Ok(Self::Minute(node.get_i64_req(ctx.consume_idx())? as u32)),
			"Hour" => Ok(Self::Hour(node.get_i64_req(ctx.consume_idx())? as u32)),
			name => Err(GeneralError(format!(
				"Invalid action activation type {name:?}, expected \
				Action, Bonus, Reaction, Special, Minute, or Hour."
			))
			.into()),
		}
	}
}
