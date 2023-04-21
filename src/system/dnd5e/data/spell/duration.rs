use crate::kdl_ext::{FromKDL, NodeContext, NodeExt};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Duration {
	pub concentration: bool,
	pub kind: DurationKind,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum DurationKind {
	#[default]
	Instantaneous,
	Unit(u64, String),
	Special,
}

impl FromKDL for Duration {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let kind = DurationKind::from_kdl(node, ctx)?;
		let concentration = node.get_bool_opt("concentration")?.unwrap_or_default();
		Ok(Self {
			concentration,
			kind,
		})
	}
}

impl FromKDL for DurationKind {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Instantaneous" => Ok(Self::Instantaneous),
			"Special" => Ok(Self::Special),
			unit => {
				let distance = node.get_i64_req(ctx.consume_idx())? as u64;
				Ok(Self::Unit(distance, unit.to_owned()))
			}
		}
	}
}
