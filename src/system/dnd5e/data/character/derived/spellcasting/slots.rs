use crate::kdl_ext::NodeContext;
use crate::{system::dnd5e::data::Rest, utility::NotInList};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub enum Slots {
	Standard {
		multiclass_half_caster: bool,
		slots_capacity: BTreeMap<usize, BTreeMap<u8, usize>>,
	},
	Bonus {
		reset_on: Rest,
		slots_capacity: BTreeMap<usize, BTreeMap<u8, usize>>,
	},
}

impl Slots {
	fn transpose_reduce_capacity(capacity: &BTreeMap<usize, BTreeMap<u8, usize>>) -> BTreeMap<u8, Vec<(usize, usize)>> {
		let mut max_amt_by_rank = [0usize; 10];
		let mut level_capacity_by_rank = BTreeMap::new();
		for (level, max_rank_slots) in capacity {
			for (rank, amt) in max_rank_slots {
				if *amt > max_amt_by_rank[*rank as usize] {
					max_amt_by_rank[*rank as usize] = *amt;
					if !level_capacity_by_rank.contains_key(rank) {
						level_capacity_by_rank.insert(*rank, Vec::new());
					}
					level_capacity_by_rank.get_mut(rank).unwrap().push((*level, *amt));
				}
			}
		}
		level_capacity_by_rank
	}

	pub fn capacity(&self) -> &BTreeMap<usize, BTreeMap<u8, usize>> {
		match self {
			Self::Standard { slots_capacity, .. } => slots_capacity,
			Self::Bonus { slots_capacity, .. } => slots_capacity,
		}
	}

	/// Returns the maximum spell rank for the provided class level.
	pub fn max_spell_rank(&self, current_level: usize) -> Option<u8> {
		let mut max_rank = None;
		for (level, rank_to_count) in self.capacity() {
			if *level > current_level {
				break;
			}
			max_rank = rank_to_count.keys().max().cloned();
		}
		max_rank
	}
}

impl FromKdl<NodeContext> for Slots {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"Standard" => {
				let multiclass_half_caster = match node.get_str_opt("multiclass")? {
					None | Some("Full") => false,
					Some("Half") => true,
					Some(name) => {
						return Err(NotInList(name.into(), vec!["Half", "Full"]).into());
					}
				};

				static MAX_LEVEL: usize = 20;
				let mut slots_capacity = (1..=MAX_LEVEL)
					.into_iter()
					.map(|level| (level, BTreeMap::new()))
					.collect::<BTreeMap<usize, BTreeMap<u8, usize>>>();
				for node in &mut node.query_all("scope() > rank")? {
					let rank = node.next_i64_req()? as u8;
					for node in &mut node.query_all("scope() > level")? {
						let level = node.next_i64_req()? as usize;
						let amount = node.next_i64_req()? as usize;
						for lvl in level..=MAX_LEVEL {
							let ranks = slots_capacity.get_mut(&lvl).unwrap();
							ranks.insert(rank, amount);
						}
					}
				}

				Ok(Self::Standard {
					multiclass_half_caster,
					slots_capacity,
				})
			}
			"Bonus" => {
				let reset_on = match node.get_str_opt("reset_on")? {
					None => Rest::Long,
					Some(str) => Rest::from_str(str)?,
				};
				let mut slots_capacity = BTreeMap::new();
				let mut prev_found_level = 0usize;
				for node in &mut node.query_all("scope() > level")? {
					let level = node.next_i64_req()? as usize;

					// Fill in unspecified levels using the last specified level data
					if prev_found_level > 0 {
						for level in (prev_found_level + 1)..level {
							let ranks = slots_capacity.get(&prev_found_level).cloned().unwrap_or_default();
							slots_capacity.insert(level, ranks);
						}
					}
					prev_found_level = level;

					// Parse the ranks for this level
					let mut ranks = BTreeMap::new();
					for node in &mut node.query_all("scope() > rank")? {
						let rank = node.next_i64_req()? as u8;
						let amount = node.next_i64_req()? as usize;
						ranks.insert(rank, amount);
					}

					slots_capacity.insert(level, ranks);
				}
				for level in (prev_found_level + 1)..=20 {
					let ranks = slots_capacity.get(&prev_found_level).cloned().unwrap_or_default();
					slots_capacity.insert(level, ranks);
				}
				Ok(Self::Bonus {
					reset_on,
					slots_capacity,
				})
			}
			name => return Err(NotInList(name.into(), vec!["Standard", "Bonus"]).into()),
		}
	}
}
impl AsKdl for Slots {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Standard {
				multiclass_half_caster,
				slots_capacity,
			} => {
				node.push_entry("Standard");
				if *multiclass_half_caster {
					node.push_entry(("multiclass", "Half"));
				}
				for (rank, capacity) in Self::transpose_reduce_capacity(slots_capacity) {
					let mut node_rank = NodeBuilder::default();
					node_rank.push_entry(rank as i64);
					for (level, amount) in capacity {
						if amount == 0 {
							continue;
						}
						node_rank.push_child(
							NodeBuilder::default()
								.with_entry(level as i64)
								.with_entry(amount as i64)
								.build("level"),
						);
					}
					node.push_child(node_rank.build("rank"));
				}
				node
			}
			Self::Bonus {
				reset_on,
				slots_capacity,
			} => {
				node.push_entry("Bonus");
				if *reset_on != Rest::Long {
					node.push_entry(("reset_on", reset_on.to_string()));
				}
				for (level, ranks) in slots_capacity {
					// Ignore reserializing any levels whose ranks match the prev level
					if *level > 1 && slots_capacity.get(&(*level - 1)) == Some(ranks) {
						continue;
					}

					let mut node_level = NodeBuilder::default();
					node_level.push_entry(*level as i64);
					for (rank, amount) in ranks {
						if *amount == 0 {
							continue;
						}
						node_level.push_child(
							NodeBuilder::default()
								.with_entry(*rank as i64)
								.with_entry(*amount as i64)
								.build("rank"),
						);
					}
					node.push_child(node_level.build("level"));
				}
				node
			}
		}
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
				|slots \"Standard\" {
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
			let data = Slots::Standard {
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
				|slots \"Standard\" multiclass=\"Half\" {
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
			let data = Slots::Standard {
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

		#[test]
		fn bonus_slots() -> anyhow::Result<()> {
			let doc = "
				|slots \"Bonus\" reset_on=\"Short\" {
				|    level 1 {
				|        rank 1 1
				|    }
				|    level 2 {
				|        rank 1 2
				|    }
				|    level 3 {
				|        rank 2 2
				|    }
				|    level 5 {
				|        rank 3 2
				|    }
				|    level 7 {
				|        rank 4 2
				|    }
				|    level 9 {
				|        rank 5 2
				|    }
				|    level 11 {
				|        rank 5 3
				|    }
				|    level 17 {
				|        rank 5 4
				|    }
				|}
			";
			let data = Slots::Bonus {
				reset_on: Rest::Short,
				slots_capacity: [
					(/*level */ 1, [(/*rank*/ 1, /*amt*/ 1)].into()),
					(/*level */ 2, [(/*rank*/ 1, /*amt*/ 2)].into()),
					(/*level */ 3, [(/*rank*/ 2, /*amt*/ 2)].into()),
					(/*level */ 4, [(/*rank*/ 2, /*amt*/ 2)].into()),
					(/*level */ 5, [(/*rank*/ 3, /*amt*/ 2)].into()),
					(/*level */ 6, [(/*rank*/ 3, /*amt*/ 2)].into()),
					(/*level */ 7, [(/*rank*/ 4, /*amt*/ 2)].into()),
					(/*level */ 8, [(/*rank*/ 4, /*amt*/ 2)].into()),
					(/*level */ 9, [(/*rank*/ 5, /*amt*/ 2)].into()),
					(/*level*/ 10, [(/*rank*/ 5, /*amt*/ 2)].into()),
					(/*level*/ 11, [(/*rank*/ 5, /*amt*/ 3)].into()),
					(/*level*/ 12, [(/*rank*/ 5, /*amt*/ 3)].into()),
					(/*level*/ 13, [(/*rank*/ 5, /*amt*/ 3)].into()),
					(/*level*/ 14, [(/*rank*/ 5, /*amt*/ 3)].into()),
					(/*level*/ 15, [(/*rank*/ 5, /*amt*/ 3)].into()),
					(/*level*/ 16, [(/*rank*/ 5, /*amt*/ 3)].into()),
					(/*level*/ 17, [(/*rank*/ 5, /*amt*/ 4)].into()),
					(/*level*/ 18, [(/*rank*/ 5, /*amt*/ 4)].into()),
					(/*level*/ 19, [(/*rank*/ 5, /*amt*/ 4)].into()),
					(/*level*/ 20, [(/*rank*/ 5, /*amt*/ 4)].into()),
				]
				.into(),
			};
			assert_eq_fromkdl!(Slots, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
