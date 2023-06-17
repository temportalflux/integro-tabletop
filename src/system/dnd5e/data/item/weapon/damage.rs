use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt},
	system::dnd5e::data::{roll::Roll, DamageType},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct WeaponDamage {
	pub roll: Option<Roll>,
	pub bonus: i32,
	pub damage_type: DamageType,
}

impl FromKDL for WeaponDamage {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let roll = match node.get_str_opt("roll")? {
			Some(roll_str) => Some(Roll::from_str(roll_str)?),
			None => None,
		};
		let base = node.get_i64_opt("base")?.unwrap_or(0) as i32;
		let damage_type = DamageType::from_str(node.get_str_req(ctx.consume_idx())?)?;
		Ok(Self {
			roll,
			bonus: base,
			damage_type,
		})
	}
}
// TODO AsKdl: WeaponDamage tests
impl AsKdl for WeaponDamage {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(self.damage_type.to_string());
		if let Some(roll) = &self.roll {
			node.push_entry(("roll", roll.to_string()));
		}
		if self.bonus != 0 {
			node.push_entry(("base", self.bonus as i64));
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{kdl_ext::NodeContext, system::dnd5e::data::roll::Die};

	fn from_doc(doc: &str) -> anyhow::Result<WeaponDamage> {
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document
			.query("scope() > damage")?
			.expect("missing damage node");
		WeaponDamage::from_kdl(node, &mut NodeContext::default())
	}

	#[test]
	fn empty() -> anyhow::Result<()> {
		let doc = "damage \"Slashing\"";
		let expected = WeaponDamage {
			roll: None,
			bonus: 0,
			damage_type: DamageType::Slashing,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}

	#[test]
	fn fixed() -> anyhow::Result<()> {
		let doc = "damage \"Slashing\" base=5";
		let expected = WeaponDamage {
			roll: None,
			bonus: 5,
			damage_type: DamageType::Slashing,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}

	#[test]
	fn roll() -> anyhow::Result<()> {
		let doc = "damage \"Slashing\" roll=\"2d4\"";
		let expected = WeaponDamage {
			roll: Some(Roll::from((2, Die::D4))),
			bonus: 0,
			damage_type: DamageType::Slashing,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}

	#[test]
	fn combined() -> anyhow::Result<()> {
		let doc = "damage \"Slashing\" roll=\"1d6\" base=2";
		let expected = WeaponDamage {
			roll: Some(Roll::from((1, Die::D6))),
			bonus: 2,
			damage_type: DamageType::Slashing,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}
}
