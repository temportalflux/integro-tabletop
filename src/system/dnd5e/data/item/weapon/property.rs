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
