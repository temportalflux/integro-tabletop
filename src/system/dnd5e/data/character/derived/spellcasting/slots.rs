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

impl FromKDL for Slots {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let multiclass_half_caster = match node.get_str_opt("multiclass")? {
			None | Some("Full") => false,
			Some("Half") => true,
			Some(name) => {
				return Err(NotInList(name.into(), vec!["Half", "Full"]).into());
			}
		};
		let reset_on = Rest::from_str(node.get_str_req("reset_on")?)?;

		static MAX_LEVEL: usize = 20;
		let mut slots_capacity = (1..=MAX_LEVEL)
			.into_iter()
			.map(|level| (level, BTreeMap::new()))
			.collect::<BTreeMap<usize, BTreeMap<u8, usize>>>();
		for mut node in &mut node.query_all("scope() > rank")? {
			let rank = node.next_i64_req()? as u8;
			for mut node in &mut node.query_all("scope() > level")? {
				let level = node.next_i64_req()? as usize;
				let amount = node.next_i64_req()? as usize;
				for lvl in level..=MAX_LEVEL {
					let ranks = slots_capacity.get_mut(&lvl).unwrap();
					ranks.insert(rank, amount);
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

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "slots";

		#[test]
		fn full_caster() -> anyhow::Result<()> {
			let doc = "
				|slots reset_on=\"Long\" {
				|    rank 1 {
				|        level 1 2
				|        level 2 3
				|        level 3 4
				|    }
				|    rank 2 {
				|        level 3 2
				|        level 4 3
				|    }
				|    rank 3 {
				|        level 5 2
				|        level 6 3
				|    }
				|    rank 4 {
				|        level 7 1
				|        level 8 2
				|        level 9 3
				|    }
				|    rank 5 {
				|        level 9 1
				|        level 10 2
				|        level 18 3
				|    }
				|    rank 6 {
				|        level 11 1
				|        level 19 2
				|    }
				|    rank 7 {
				|        level 13 1
				|        level 20 2
				|    }
				|    rank 8 {
				|        level 15 1
				|    }
				|    rank 9 {
				|        level 17 1
				|    }
				|}
			";
			let data = Slots {
				reset_on: Rest::Long,
				multiclass_half_caster: false,
				slots_capacity: [
					(/*level*/ 1, [(/*rank*/ 1, /*amt*/ 2)].into()),
					(/*level*/ 2, [(/*rank*/ 1, /*amt*/ 3)].into()),
					(
						/*level*/ 3,
						[(/*rank*/ 1, /*amt*/ 4), (/*rank*/ 2, /*amt*/ 2)].into(),
					),
					(
						/*level*/ 4,
						[(/*rank*/ 1, /*amt*/ 4), (/*rank*/ 2, /*amt*/ 3)].into(),
					),
					(
						/*level*/ 5,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 6,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
						]
						.into(),
					),
					(
						/*level*/ 7,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 8,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 9,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 10,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 11,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 12,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 13,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
							(/*rank*/ 7, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 14,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
							(/*rank*/ 7, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 15,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
							(/*rank*/ 7, /*amt*/ 1),
							(/*rank*/ 8, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 16,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
							(/*rank*/ 7, /*amt*/ 1),
							(/*rank*/ 8, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 17,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
							(/*rank*/ 6, /*amt*/ 1),
							(/*rank*/ 7, /*amt*/ 1),
							(/*rank*/ 8, /*amt*/ 1),
							(/*rank*/ 9, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 18,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 3),
							(/*rank*/ 6, /*amt*/ 1),
							(/*rank*/ 7, /*amt*/ 1),
							(/*rank*/ 8, /*amt*/ 1),
							(/*rank*/ 9, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 19,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 3),
							(/*rank*/ 6, /*amt*/ 2),
							(/*rank*/ 7, /*amt*/ 1),
							(/*rank*/ 8, /*amt*/ 1),
							(/*rank*/ 9, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 20,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 3),
							(/*rank*/ 6, /*amt*/ 2),
							(/*rank*/ 7, /*amt*/ 2),
							(/*rank*/ 8, /*amt*/ 1),
							(/*rank*/ 9, /*amt*/ 1),
						]
						.into(),
					),
				]
				.into(),
			};
			assert_eq_fromkdl!(Slots, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn half_caster() -> anyhow::Result<()> {
			let doc = "
				|slots multiclass=\"Half\" reset_on=\"Short\" {
				|    rank 1 {
				|        level 2 2
				|        level 3 3
				|        level 5 4
				|    }
				|    rank 2 {
				|        level 5 2
				|        level 7 3
				|    }
				|    rank 3 {
				|        level 9 2
				|        level 11 3
				|    }
				|    rank 4 {
				|        level 13 1
				|        level 15 2
				|        level 17 3
				|    }
				|    rank 5 {
				|        level 17 1
				|        level 19 2
				|    }
				|}
			";
			let data = Slots {
				reset_on: Rest::Short,
				multiclass_half_caster: true,
				slots_capacity: [
					(/*level*/ 1, [].into()),
					(/*level*/ 2, [(/*rank*/ 1, /*amt*/ 2)].into()),
					(/*level*/ 3, [(/*rank*/ 1, /*amt*/ 3)].into()),
					(/*level*/ 4, [(/*rank*/ 1, /*amt*/ 3)].into()),
					(
						/*level*/ 5,
						[(/*rank*/ 1, /*amt*/ 4), (/*rank*/ 2, /*amt*/ 2)].into(),
					),
					(
						/*level*/ 6,
						[(/*rank*/ 1, /*amt*/ 4), (/*rank*/ 2, /*amt*/ 2)].into(),
					),
					(
						/*level*/ 7,
						[(/*rank*/ 1, /*amt*/ 4), (/*rank*/ 2, /*amt*/ 3)].into(),
					),
					(
						/*level*/ 8,
						[(/*rank*/ 1, /*amt*/ 4), (/*rank*/ 2, /*amt*/ 3)].into(),
					),
					(
						/*level*/ 9,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 10,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 11,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
						]
						.into(),
					),
					(
						/*level*/ 12,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
						]
						.into(),
					),
					(
						/*level*/ 13,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 14,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 15,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 16,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 17,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 18,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 1),
						]
						.into(),
					),
					(
						/*level*/ 19,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
						]
						.into(),
					),
					(
						/*level*/ 20,
						[
							(/*rank*/ 1, /*amt*/ 4),
							(/*rank*/ 2, /*amt*/ 3),
							(/*rank*/ 3, /*amt*/ 3),
							(/*rank*/ 4, /*amt*/ 3),
							(/*rank*/ 5, /*amt*/ 2),
						]
						.into(),
					),
				]
				.into(),
			};
			assert_eq_fromkdl!(Slots, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
