use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
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

impl Slots {
	fn transpose_reduce_capacity(&self) -> BTreeMap<u8, Vec<(usize, usize)>> {
		let mut max_amt_by_rank = [0usize; 10];
		let mut level_capacity_by_rank = BTreeMap::new();
		for (level, max_rank_slots) in &self.slots_capacity {
			for (rank, amt) in max_rank_slots {
				if *amt > max_amt_by_rank[*rank as usize] {
					max_amt_by_rank[*rank as usize] = *amt;
					if !level_capacity_by_rank.contains_key(rank) {
						level_capacity_by_rank.insert(*rank, Vec::new());
					}
					level_capacity_by_rank
						.get_mut(rank)
						.unwrap()
						.push((*level, *amt));
				}
			}
		}
		level_capacity_by_rank
	}
}

// TODO AsKdl: from/as for spellcasting Slots
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
impl AsKdl for Slots {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if self.multiclass_half_caster {
			node.push_entry(("multiclass", "Half"));
		}
		node.push_entry(("reset_on", self.reset_on.to_string()));

		for (rank, capacity) in self.transpose_reduce_capacity() {
			let mut node_rank = NodeBuilder::default();
			node_rank.push_entry(rank as i64);
			for (level, amt) in capacity {
				node_rank.push_child(
					NodeBuilder::default()
						.with_entry(level as i64)
						.with_entry(amt as i64)
						.build("level"),
				);
			}
			node.push_child(node_rank.build("rank"));
		}

		node
	}
}
