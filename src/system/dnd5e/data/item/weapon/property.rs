use crate::{
	kdl_ext::{FromKDL, NodeContext, NodeExt},
	system::dnd5e::data::roll::Roll,
	GeneralError,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum Property {
	Light,   // used by two handed fighting feature
	Finesse, // melee weapons use strength, ranged use dex, finesse take the better of either modifier
	Heavy,   // small or tiny get disadvantage on attack rolls when using this weapon
	Reach, // This weapon adds 5 feet to your reach when you attack with it, as well as when determining your reach for opportunity attacks with it.
	TwoHanded,
	Thrown(u32, u32),
	Versatile(Roll),
}

impl Property {
	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Light => "Light",
			Self::Finesse => "Finesse",
			Self::Heavy => "Heavy",
			Self::Reach => "Reach",
			Self::TwoHanded => "Two Handed",
			Self::Thrown(_, _) => "Thrown",
			Self::Versatile(_) => "Versatile",
		}
	}

	pub fn description(&self) -> String {
		match self {
			Self::Light => {
				"When you use the Attack action to make a melee attack with this weapon, \
				you can use a bonus action to attack with a different light melee weapon \
				that you're holding in the other hand."
					.into()
			}
			Self::Finesse => "You can use either your Strength or Dexterity modifier \
				for both the attack and damage rolls."
				.into(),
			Self::Heavy => {
				"Small or Tiny creatures have disadvantage on attack rolls with this weapon.".into()
			}
			Self::Reach => "This weapon extends an additional 5 feet of melee range when \
				making the attack action or opportunity attacks."
				.into(),
			Self::TwoHanded => "This weapon requires two hands when you attack with it.".into(),
			Self::Thrown(min, max) => format!(
				"You can throw this weapon to make a ranged attack, \
				with an inner-range of {min} and an outer-range of {max}."
			),
			Self::Versatile(roll) => format!(
				"This weapon can be used with one or two hands. \
				You deal {} damage when using two hands.",
				roll.to_string()
			),
		}
	}
}

impl FromKDL for Property {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Light" => Ok(Self::Light),
			"Finesse" => Ok(Self::Finesse),
			"Heavy" => Ok(Self::Heavy),
			"Reach" => Ok(Self::Reach),
			"TwoHanded" => Ok(Self::TwoHanded),
			"Thrown" => {
				let short = node.get_i64_req(ctx.consume_idx())? as u32;
				let long = node.get_i64_req(ctx.consume_idx())? as u32;
				Ok(Self::Thrown(short, long))
			}
			"Versatile" => {
				let roll = Roll::from_str(node.get_str_req(ctx.consume_idx())?)?;
				Ok(Self::Versatile(roll))
			}
			name => Err(GeneralError(format!("Unrecognized weapon property {name:?}")).into()),
		}
	}
}
