use super::super::{AreaOfEffect, DamageRoll};
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{core::NodeRegistry, dnd5e::FromKDL},
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

impl FromKDL for Attack {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let kind =
			AttackKindValue::from_kdl(node.query_req("kind")?, &mut ValueIdx::default(), node_reg)?;
		let check = AttackCheckKind::from_kdl(
			node.query_req("check")?,
			&mut ValueIdx::default(),
			node_reg,
		)?;
		let area_of_effect = match node.query("area_of_effect")? {
			None => None,
			Some(node) => Some(AreaOfEffect::from_kdl(
				node,
				&mut ValueIdx::default(),
				node_reg,
			)?),
		};
		let damage = match node.query("damage")? {
			None => None,
			Some(node) => Some(DamageRoll::from_kdl(
				node,
				&mut ValueIdx::default(),
				node_reg,
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
