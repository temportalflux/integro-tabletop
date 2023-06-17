use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt, ValueExt},
	utility::NotInList,
};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum Range {
	#[default]
	OnlySelf,
	Touch,
	Unit {
		distance: u64,
		unit: String,
	},
	Sight,
	Unlimited,
}

impl FromKDL for Range {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let entry = node.entry_req(ctx.consume_idx())?;
		match entry.value().as_string() {
			None => {
				let distance = entry.as_i64_req()? as u64;
				let unit = node
					.get_str_opt(ctx.consume_idx())?
					.unwrap_or("Feet")
					.to_owned();
				Ok(Self::Unit { distance, unit })
			}
			Some("Self") => Ok(Self::OnlySelf),
			Some("Touch") => Ok(Self::Touch),
			Some("Sight") => Ok(Self::Sight),
			Some("Unlimited") => Ok(Self::Unlimited),
			Some(type_name) => Err(NotInList(
				type_name.into(),
				vec!["Self", "Touch", "Sight", "Unlimited"],
			)
			.into()),
		}
	}
}
// TODO AsKdl: from/as tests for spell Range
impl AsKdl for Range {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::OnlySelf => node.with_entry("Self"),
			Self::Touch => node.with_entry("Touch"),
			Self::Sight => node.with_entry("Sight"),
			Self::Unlimited => node.with_entry("Unlimited"),
			Self::Unit { distance, unit } => {
				node.push_entry(*distance as i64);
				if unit != "Feet" {
					node.push_entry(unit.clone());
				}
				node
			}
		}
	}
}
