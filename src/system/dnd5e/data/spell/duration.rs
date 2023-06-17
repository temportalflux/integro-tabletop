use crate::kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt};

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
// TODO AsKdl: from/as tests for spell duration/kind
impl AsKdl for Duration {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.kind.as_kdl();
		if self.concentration {
			node.push_entry(("concentration", true));
		}
		node
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
impl AsKdl for DurationKind {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Instantaneous => node.with_entry("Instantaneous"),
			Self::Special => node.with_entry("Special"),
			Self::Unit(distance, unit) => {
				node.with_entry(unit.clone()).with_entry(*distance as i64)
			}
		}
	}
}

impl DurationKind {
	pub fn as_metadata(&self) -> String {
		match self {
			Self::Instantaneous => "instant",
			Self::Special | Self::Unit(_, _) => "other",
		}
		.into()
	}
}
