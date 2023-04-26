use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::Rest,
	utility::NotInList,
};
use std::{collections::BTreeMap, str::FromStr};

pub struct Slots {
	multiclass_half_caster: bool,
	reset_on: Rest,
	slots_capacity: BTreeMap<u8, BTreeMap<usize, usize>>,
}

impl FromKDL for Slots {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let multiclass_half_caster = match node.get_str_opt("multiclass")? {
			None | Some("Full") => false,
			Some("Half") => true,
			Some(name) => {
				return Err(NotInList(name.into(), vec!["Half", "Full"]).into());
			}
		};
		let reset_on = Rest::from_str(node.get_str_req("reset_on")?)?;

		let mut slots_by_rank = BTreeMap::new();
		for node in node.query_all("scope() > rank")? {
			let mut ctx = ctx.next_node();
			let rank = node.get_i64_req(ctx.consume_idx())? as u8;
			let mut amt_by_level = BTreeMap::new();
			for node in node.query_all("scope() > level")? {
				let mut ctx = ctx.next_node();
				let level = node.get_i64_req(ctx.consume_idx())? as usize;
				let amount = node.get_i64_req(ctx.consume_idx())? as usize;
				amt_by_level.insert(level, amount);
			}
			slots_by_rank.insert(rank, amt_by_level);
		}

		Ok(Self {
			multiclass_half_caster,
			reset_on,
			slots_capacity: slots_by_rank,
		})
	}
}
