use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt},
	system::dnd5e::data::{
		character::Character,
		roll::{Roll, RollSet},
		scaling, DamageType,
	},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct Damage {
	amount: scaling::Value<Roll>,
	// amount to add to the rolled damage value
	base: i32,
	// if true, add the spellcasting ability modifier to the total damage
	include_ability_modifier: bool,
	damage_type: DamageType,
	upcast: Option<Roll>,
}

impl Damage {
	pub fn evaluate(
		&self,
		character: &Character,
		modifier: i32,
		upcast_amount: u32,
	) -> (RollSet, i32) {
		let mut rolls = RollSet::default();
		if let Some(roll) = self.amount.evaluate(character) {
			rolls.push(roll);
		}
		if let Some(upcast_roll) = &self.upcast {
			if upcast_amount > 0 {
				rolls.extend(RollSet::multiple(upcast_roll, upcast_amount));
			}
		}
		let mut bonus = self.base;
		if self.include_ability_modifier {
			bonus += modifier;
		}
		(rolls, bonus)
	}
}

impl FromKDL for Damage {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let amount = scaling::Value::from_kdl(node, ctx)?;
		let damage_type = DamageType::from_str(node.get_str_req(ctx.consume_idx())?)?;
		let base = node.get_i64_opt("base")?.unwrap_or_default() as i32;
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
// TODO AsKdl: from/as tests for spell Damage
impl AsKdl for Damage {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.amount.as_kdl();
		node.push_entry(self.damage_type.display_name());
		if self.base != 0 {
			node.push_entry(("base", self.base as i64));
		}
		if self.include_ability_modifier {
			node.push_entry(("ability", true));
		}
		if let Some(upcast) = &self.upcast {
			node.push_entry(("upcast", upcast.to_string()));
		}
		node
	}
}
