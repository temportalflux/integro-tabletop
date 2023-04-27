use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::Rest,
	utility::NotInList,
};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct Slots {
	pub multiclass_half_caster: bool,
	pub reset_on: Rest,
	pub slots_capacity: BTreeMap<usize, BTreeMap<u8, usize>>,
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

		let mut slots_capacity: BTreeMap<usize, BTreeMap<u8, usize>> = Default::default();
		let mut max_rank_slots = BTreeMap::new();
		for node in node.query_all("scope() > rank")? {
			let mut ctx = ctx.next_node();
			let rank = node.get_i64_req(ctx.consume_idx())? as u8;
			for node in node.query_all("scope() > level")? {
				let mut ctx = ctx.next_node();
				let level = node.get_i64_req(ctx.consume_idx())? as usize;
				let amount = node.get_i64_req(ctx.consume_idx())? as usize;
				max_rank_slots.insert(rank, amount);
				if let Some(ranks) = slots_capacity.get_mut(&level) {
					ranks.insert(rank, amount);
				} else {
					slots_capacity.insert(level, max_rank_slots.clone());
				}
			}
		}

		Ok(Self {
			multiclass_half_caster,
			reset_on,
			slots_capacity,
		})
	}
}
