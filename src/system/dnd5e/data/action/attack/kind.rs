use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	utility::NotInList,
};
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttackKind {
	Melee,
	Ranged,
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
// TODO AsKdl: tests for AttackKindValue
impl AsKdl for AttackKindValue {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
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

	mod from_kdl {
		use super::*;
		use crate::kdl_ext::NodeContext;

		fn from_doc(doc: &str) -> anyhow::Result<AttackKindValue> {
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > kind")?
				.expect("missing kind node");
			AttackKindValue::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn melee_base() -> anyhow::Result<()> {
			let doc = "kind \"Melee\"";
			let expected = AttackKindValue::Melee { reach: 5 };
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn melee_reach() -> anyhow::Result<()> {
			let doc = "kind \"Melee\" reach=10";
			let expected = AttackKindValue::Melee { reach: 10 };
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn ranged_base() -> anyhow::Result<()> {
			let doc = "kind \"Ranged\" 20 60";
			let expected = AttackKindValue::Ranged {
				short_dist: 20,
				long_dist: 60,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
