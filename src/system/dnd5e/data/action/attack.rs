use super::super::{AreaOfEffect, DamageRoll};
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	system::dnd5e::data::item::weapon,
};

mod check;
pub use check::*;
mod kind;
pub use kind::*;
mod range;
pub use range::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Attack {
	pub kind: Option<AttackKindValue>,
	pub check: AttackCheckKind,
	pub area_of_effect: Option<AreaOfEffect>,
	pub damage: Option<DamageRoll>,
	pub weapon_kind: Option<weapon::Kind>,
}

impl FromKDL for Attack {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let kind = match node.query_opt("scope() > kind")? {
			None => None,
			Some(node) => Some(AttackKindValue::from_kdl(node, &mut ctx.next_node())?),
		};
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
			weapon_kind: None,
		})
	}
}

impl AsKdl for Attack {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(kind) = &self.kind {
			node.push_child_t("kind", kind);
		}
		node.push_child_t("check", &self.check);
		if let Some(area_of_effect) = &self.area_of_effect {
			node.push_child_t("area_of_effect", area_of_effect);
		}
		if let Some(damage) = &self.damage {
			node.push_child_t("damage", damage);
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::dnd5e::data::{
				roll::{Die, EvaluatedRoll},
				Ability, DamageType,
			},
			utility,
		};

		static NODE_NAME: &str = "attack";

		#[test]
		fn melee_attackroll_damage() -> anyhow::Result<()> {
			let doc = "
				|attack {
				|    kind \"Melee\"
				|    check \"AttackRoll\" (Ability)\"Dexterity\" proficient=true
				|    damage base=1 {
				|        roll (Roll)\"2d6\"
				|        damage_type \"Fire\"
				|    }
				|}
			";
			let data = Attack {
				kind: Some(AttackKindValue::Melee { reach: 5 }),
				check: AttackCheckKind::AttackRoll {
					ability: Ability::Dexterity,
					proficient: utility::Value::Fixed(true),
				},
				area_of_effect: None,
				damage: Some(DamageRoll {
					roll: Some(EvaluatedRoll::from((2, Die::D6))),
					base_bonus: 1,
					damage_type: DamageType::Fire,
					additional_bonuses: Vec::new(),
				}),
				weapon_kind: None,
			};
			assert_eq_fromkdl!(Attack, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn ranged_savingthrow_aoe_damage() -> anyhow::Result<()> {
			let doc = "
				|attack {
				|    kind \"Ranged\" 20 60
				|    check \"SavingThrow\" {
				|        difficulty_class 8
				|        save_ability (Ability)\"Constitution\"
				|    }
				|    area_of_effect \"Sphere\" radius=10
				|    damage base=1 {
				|        roll (Roll)\"2d6\"
				|        damage_type \"Fire\"
				|    }
				|}
			";
			let data = Attack {
				kind: Some(AttackKindValue::Ranged {
					short_dist: 20,
					long_dist: 60,
				}),
				check: AttackCheckKind::SavingThrow {
					base: 8,
					dc_ability: None,
					proficient: false,
					save_ability: Ability::Constitution,
				},
				area_of_effect: Some(AreaOfEffect::Sphere { radius: 10 }),
				damage: Some(DamageRoll {
					roll: Some(EvaluatedRoll::from((2, Die::D6))),
					base_bonus: 1,
					damage_type: DamageType::Fire,
					additional_bonuses: Vec::new(),
				}),
				weapon_kind: None,
			};
			assert_eq_fromkdl!(Attack, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
