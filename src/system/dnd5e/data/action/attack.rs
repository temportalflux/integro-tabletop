use super::super::{AreaOfEffect, DamageRoll};
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{DnD5e, FromKDL},
};

mod check;
pub use check::*;
mod kind;
pub use kind::*;
mod range;
pub use range::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Attack {
	pub kind: AttackKindValue,
	pub check: AttackCheckKind,
	pub area_of_effect: Option<AreaOfEffect>,
	pub damage: Option<DamageRoll>,
}

impl FromKDL<DnD5e> for Attack {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let kind =
			AttackKindValue::from_kdl(node.query_req("kind")?, &mut ValueIdx::default(), system)?;
		let check =
			AttackCheckKind::from_kdl(node.query_req("check")?, &mut ValueIdx::default(), system)?;
		let area_of_effect = match node.query("area_of_effect")? {
			None => None,
			Some(node) => Some(AreaOfEffect::from_kdl(
				node,
				&mut ValueIdx::default(),
				system,
			)?),
		};
		let damage = match node.query("damage")? {
			None => None,
			Some(node) => Some(DamageRoll::from_kdl(
				node,
				&mut ValueIdx::default(),
				system,
			)?),
		};
		Ok(Self {
			kind,
			check,
			area_of_effect,
			damage,
		})
	}
}
