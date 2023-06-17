use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt},
	system::dnd5e::data::{action::AttackKind, Ability},
	utility::NotInList,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum Check {
	AttackRoll(AttackKind),
	SavingThrow(Ability, Option<u8>),
}

impl FromKDL for Check {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"AttackRoll" => Ok(Self::AttackRoll(AttackKind::from_str(
				node.get_str_req(ctx.consume_idx())?,
			)?)),
			"SavingThrow" => {
				let ability = Ability::from_str(node.get_str_req(ctx.consume_idx())?)?;
				let dc = node.get_i64_opt("dc")?.map(|v| v as u8);
				Ok(Self::SavingThrow(ability, dc))
			}
			name => Err(NotInList(name.into(), vec!["AttackRoll", "SavingThrow"]).into()),
		}
	}
}
// TODO AsKdl: from/as tests for spell checks
impl AsKdl for Check {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::AttackRoll(kind) => {
				NodeBuilder::default()
					.with_entry("AttackRoll")
					.with_entry(match kind {
						AttackKind::Melee => "Melee",
						AttackKind::Ranged => "Ranged",
					})
			}
			Self::SavingThrow(ability, dc) => {
				let mut node = NodeBuilder::default().with_entry("SavingThrow");
				node.push_entry({
					let mut entry = kdl::KdlEntry::new(ability.long_name());
					entry.set_ty("Ability");
					entry
				});
				if let Some(dc) = dc {
					node.push_entry(("dc", *dc as i64));
				}
				node
			}
		}
	}
}
