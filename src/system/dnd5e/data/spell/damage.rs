use crate::kdl_ext::NodeContext;
use crate::system::dnd5e::data::{
	character::Character,
	roll::{Roll, RollSet},
	scaling, DamageType,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
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
	pub fn evaluate(&self, character: &Character, modifier: i32, upcast_amount: u32) -> (RollSet, i32) {
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

impl FromKdl<NodeContext> for Damage {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let amount = scaling::Value::from_kdl(node)?;
		let damage_type = node.next_str_req_t::<DamageType>()?;
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

impl AsKdl for Damage {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.amount.as_kdl();
		node.push_entry_typed(self.damage_type.display_name(), "DamageType");
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

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::data::roll::Die};

		static NODE_NAME: &str = "damage";

		#[test]
		fn fixed_roll() -> anyhow::Result<()> {
			let doc = "damage \"2d6\" (DamageType)\"Force\"";
			let data = Damage {
				amount: scaling::Value::Fixed(Roll::from((2, Die::D6))),
				damage_type: DamageType::Force,
				base: 0,
				include_ability_modifier: false,
				upcast: None,
			};
			assert_eq_fromkdl!(Damage, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn varying_roll() -> anyhow::Result<()> {
			let doc = "damage (Scaled)\"Level\" (DamageType)\"Force\"";
			let data = Damage {
				amount: scaling::Value::Scaled(scaling::Basis::Level {
					class_name: None,
					level_map: [].into(),
				}),
				damage_type: DamageType::Force,
				base: 0,
				include_ability_modifier: false,
				upcast: None,
			};
			assert_eq_fromkdl!(Damage, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn with_base() -> anyhow::Result<()> {
			let doc = "damage \"2d6\" (DamageType)\"Force\" base=2";
			let data = Damage {
				amount: scaling::Value::Fixed(Roll::from((2, Die::D6))),
				damage_type: DamageType::Force,
				base: 2,
				include_ability_modifier: false,
				upcast: None,
			};
			assert_eq_fromkdl!(Damage, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn with_ability_mod() -> anyhow::Result<()> {
			let doc = "damage \"2d6\" (DamageType)\"Force\" ability=true";
			let data = Damage {
				amount: scaling::Value::Fixed(Roll::from((2, Die::D6))),
				damage_type: DamageType::Force,
				base: 0,
				include_ability_modifier: true,
				upcast: None,
			};
			assert_eq_fromkdl!(Damage, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn with_upcast() -> anyhow::Result<()> {
			let doc = "damage \"2d6\" (DamageType)\"Force\" upcast=\"1d6\"";
			let data = Damage {
				amount: scaling::Value::Fixed(Roll::from((2, Die::D6))),
				damage_type: DamageType::Force,
				base: 0,
				include_ability_modifier: false,
				upcast: Some(Roll::from((1, Die::D6))),
			};
			assert_eq_fromkdl!(Damage, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
