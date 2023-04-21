use std::str::FromStr;

use crate::{
	kdl_ext::{FromKDL, NodeContext, NodeExt},
	system::dnd5e::data::{roll::Roll, scaling, DamageType},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Damage {
	amount: scaling::Value<Roll>,
	// amount to add to the rolled damage value
	base: u64,
	// if true, add the spellcasting ability modifier to the total damage
	include_ability_modifier: bool,
	damage_type: DamageType,
	upcast: Option<Roll>,
}

impl FromKDL for Damage {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let amount = scaling::Value::from_kdl(node, ctx)?;
		let damage_type = DamageType::from_str(node.get_str_req(ctx.consume_idx())?)?;
		let base = node.get_i64_opt("base")?.unwrap_or_default() as u64;
		let ability = node.get_bool_opt("ability")?.unwrap_or_default();
		let upcast = match node.get_str_opt("upcast")? {
			None => None,
			Some(str) => Some(Roll::from_str(str)?),
		};
		Ok(Self {
			amount,
			base,
			include_ability_modifier: ability,
			damage_type,
			upcast,
		})
	}
}
