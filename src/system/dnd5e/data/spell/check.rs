use crate::{
	kdl_ext::{FromKDL, NodeContext, NodeExt},
	system::dnd5e::data::{action::AttackKind, Ability},
	utility::NotInList,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum Check {
	AttackRoll(AttackKind),
	SavingThrow(Ability),
}

impl FromKDL for Check {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"AttackRoll" => Ok(Self::AttackRoll(AttackKind::from_str(
				node.get_str_req(ctx.consume_idx())?,
			)?)),
			"SavingThrow" => Ok(Self::SavingThrow(Ability::from_str(
				node.get_str_req(ctx.consume_idx())?,
			)?)),
			name => Err(NotInList(name.into(), vec!["AttackRoll", "SavingThrow"]).into()),
		}
	}
}
