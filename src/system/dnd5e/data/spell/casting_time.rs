use crate::kdl_ext::{FromKDL, NodeContext, NodeExt};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct CastingTime {
	pub duration: CastingDuration,
	pub ritual: bool,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum CastingDuration {
	#[default]
	Action,
	Bonus,
	Reaction(Option<String>),
	Unit(u64, String),
}

impl FromKDL for CastingTime {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let duration = CastingDuration::from_kdl(node, ctx)?;
		let ritual = node.get_bool_opt("ritual")?.unwrap_or_default();
		Ok(Self { duration, ritual })
	}
}

impl FromKDL for CastingDuration {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction(
				node.get_str_opt(ctx.consume_idx())?.map(str::to_owned),
			)),
			unit => Ok(Self::Unit(
				node.get_i64_req(ctx.consume_idx())? as u64,
				unit.to_owned(),
			)),
		}
	}
}

impl CastingDuration {
	pub fn as_metadata(&self) -> String {
		match self {
			Self::Action => "action",
			Self::Bonus => "bonus",
			Self::Reaction(_) => "reaction",
			Self::Unit(_, _) => "other",
		}
		.into()
	}
}
