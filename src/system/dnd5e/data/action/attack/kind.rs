use super::RangeKind;
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{DnD5e, FromKDL},
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

impl FromKDL<DnD5e> for AttackKindValue {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"Melee" => Ok(Self::Melee {
				reach: node.get_i64_opt("reach")?.unwrap_or(5) as u32,
			}),
			"Ranged" => {
				let short_dist = node.get_i64(value_idx.next())? as u32;
				let long_dist = node.get_i64(value_idx.next())? as u32;
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
