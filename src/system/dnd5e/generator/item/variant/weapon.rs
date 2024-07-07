use crate::{kdl_ext::NodeContext, system::dnd5e::data::item::equipment::Equipment};
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct WeaponExtension {
	atk_bonus: i32,
	dmg_bonus: i32,
}

impl WeaponExtension {
	pub fn apply_to(&self, equipment: &mut Equipment) -> anyhow::Result<()> {
		if let Some(weapon) = &mut equipment.weapon {
			weapon.attack_roll_bonus += self.atk_bonus;
			if let Some(damage) = &mut weapon.damage {
				damage.bonus += self.dmg_bonus;
			}
		}
		Ok(())
	}
}

impl FromKdl<NodeContext> for WeaponExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let atk_bonus = node.query_i64_opt("scope() > attack-roll", "bonus")?.unwrap_or_default() as i32;
		let dmg_bonus = node.query_i64_opt("scope() > attack-damage", "bonus")?.unwrap_or_default() as i32;
		Ok(Self { atk_bonus, dmg_bonus })
	}
}

impl AsKdl for WeaponExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if self.atk_bonus != 0 {
			node.child(("attack-roll", NodeBuilder::default().with_entry(("bonus", self.atk_bonus as i64))));
		}
		if self.dmg_bonus != 0 {
			node.child(("attack-damage", NodeBuilder::default().with_entry(("bonus", self.atk_bonus as i64))));
		}

		node
	}
}
