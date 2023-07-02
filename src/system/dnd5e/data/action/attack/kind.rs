use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	utility::NotInList,
};
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(EnumSetType, PartialOrd, Ord, Hash, Debug)]
pub enum AttackKind {
	Melee,
	Ranged,
}

impl ToString for AttackKind {
	fn to_string(&self) -> String {
		match self {
			Self::Melee => "Melee",
			Self::Ranged => "Ranged",
		}
		.into()
	}
}

impl FromStr for AttackKind {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Melee" => Ok(Self::Melee),
			"Ranged" => Ok(Self::Ranged),
			_ => Err(NotInList(s.into(), vec!["Melee", "Ranged"]).into()),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackKindValue {
	Melee { reach: u32 },
	Ranged { short_dist: u32, long_dist: u32 },
}

impl AttackKindValue {
	pub fn kind(&self) -> AttackKind {
		match self {
			Self::Melee { .. } => AttackKind::Melee,
			Self::Ranged { .. } => AttackKind::Ranged,
		}
	}
}

impl FromKDL for AttackKindValue {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Melee" => Ok(Self::Melee {
				reach: node.get_i64_opt("reach")?.unwrap_or(5) as u32,
			}),
			"Ranged" => {
				let short_dist = node.get_i64_req(ctx.consume_idx())? as u32;
				let long_dist = node.get_i64_req(ctx.consume_idx())? as u32;
				Ok(Self::Ranged {
					short_dist,
					long_dist,
				})
			}
			name => Err(NotInList(name.into(), vec!["Melee", "Ranged"]).into()),
		}
	}
}

impl AsKdl for AttackKindValue {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Melee { reach } => {
				let mut node = node.with_entry("Melee");
				if *reach != 5 {
					node.push_entry(("reach", *reach as i64));
				}
				node
			}
			Self::Ranged {
				short_dist,
				long_dist,
			} => node
				.with_entry("Ranged")
				.with_entry(*short_dist as i64)
				.with_entry(*long_dist as i64),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "kind";

		#[test]
		fn melee_base() -> anyhow::Result<()> {
			let doc = "kind \"Melee\"";
			let data = AttackKindValue::Melee { reach: 5 };
			assert_eq_fromkdl!(AttackKindValue, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn melee_reach() -> anyhow::Result<()> {
			let doc = "kind \"Melee\" reach=10";
			let data = AttackKindValue::Melee { reach: 10 };
			assert_eq_fromkdl!(AttackKindValue, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn ranged_base() -> anyhow::Result<()> {
			let doc = "kind \"Ranged\" 20 60";
			let data = AttackKindValue::Ranged {
				short_dist: 20,
				long_dist: 60,
			};
			assert_eq_fromkdl!(AttackKindValue, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
