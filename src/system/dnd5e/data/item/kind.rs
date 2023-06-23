use super::equipment::Equipment;
use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt},
	utility::NotInList,
};

#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
	Simple { count: u32 },
	Equipment(Equipment),
}

impl Default for Kind {
	fn default() -> Self {
		Self::Simple { count: 1 }
	}
}

impl FromKDL for Kind {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Simple" => {
				let count = node.get_i64_opt("count")?.unwrap_or(1) as u32;
				Ok(Self::Simple { count })
			}
			"Equipment" => {
				let equipment = Equipment::from_kdl(node, ctx)?;
				Ok(Self::Equipment(equipment))
			}
			value => Err(NotInList(value.into(), vec!["Simple", "Equipment"]).into()),
		}
	}
}

impl AsKdl for Kind {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Simple { count } => {
				node.push_entry("Simple");
				if *count > 1 {
					node.push_entry(("count", *count as i64));
				}
			}
			Self::Equipment(equipment) => {
				node.push_entry("Equipment");
				node += equipment.as_kdl();
			}
		}
		node
	}
}
