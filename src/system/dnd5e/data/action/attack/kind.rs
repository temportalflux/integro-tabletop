use super::RangeKind;
use crate::{
	kdl_ext::{NodeExt, ValueIdx},
	system::{core::NodeRegistry, dnd5e::FromKDL},
	GeneralError,
};
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AttackKind {
	Melee,
	Ranged,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackKindValue {
	Melee {
		reach: u32,
	},
	Ranged {
		short_dist: u32,
		long_dist: u32,
		kind: Option<RangeKind>,
	},
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
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		match node.get_str_req(value_idx.next())? {
			"Melee" => Ok(Self::Melee {
				reach: node.get_i64_opt("reach")?.unwrap_or(5) as u32,
			}),
			"Ranged" => {
				let short_dist = node.get_i64_req(value_idx.next())? as u32;
				let long_dist = node.get_i64_req(value_idx.next())? as u32;
				let kind = match node.get_str_opt("kind")? {
					None => None,
					Some(str) => Some(RangeKind::from_str(str)?),
				};
				Ok(Self::Ranged {
					short_dist,
					long_dist,
					kind,
				})
			}
			name => Err(GeneralError(format!(
				"Invalid attack kind {name:?}, expected Melee or Ranged."
			))
			.into()),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::core::NodeRegistry;

		fn from_doc(doc: &str) -> anyhow::Result<AttackKindValue> {
			let node_reg = NodeRegistry::default();
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > kind")?
				.expect("missing kind node");
			let mut idx = ValueIdx::default();
			AttackKindValue::from_kdl(node, &mut idx, &node_reg)
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
				kind: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn ranged_kind() -> anyhow::Result<()> {
			let doc = "kind \"Ranged\" 20 60 kind=\"Unlimited\"";
			let expected = AttackKindValue::Ranged {
				short_dist: 20,
				long_dist: 60,
				kind: Some(RangeKind::Unlimited),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
