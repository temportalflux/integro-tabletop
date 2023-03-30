use super::super::{AreaOfEffect, DamageRoll};
use crate::kdl_ext::{DocumentExt, FromKDL};

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
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let kind =
			AttackKindValue::from_kdl(node.query_req("scope() > kind")?, &mut ctx.next_node())?;
		let check =
			AttackCheckKind::from_kdl(node.query_req("scope() > check")?, &mut ctx.next_node())?;
		let area_of_effect = match node.query("scope() > area_of_effect")? {
			None => None,
			Some(node) => Some(AreaOfEffect::from_kdl(node, &mut ctx.next_node())?),
		};
		let damage = match node.query("scope() > damage")? {
			None => None,
			Some(node) => Some(DamageRoll::from_kdl(node, &mut ctx.next_node())?),
		};
		Ok(Self {
			kind,
			check,
			area_of_effect,
			damage,
		})
	}
}
