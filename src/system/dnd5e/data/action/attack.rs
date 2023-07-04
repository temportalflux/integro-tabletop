use super::super::{AreaOfEffect, DamageRoll};
use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
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
	pub properties: Vec<weapon::Property>,
}

impl FromKDL for Attack {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = node.query_opt_t::<AttackKindValue>("scope() > kind")?;
		let check = node.query_req_t::<AttackCheckKind>("scope() > check")?;
		let area_of_effect = node.query_opt_t::<AreaOfEffect>("scope() > area_of_effect")?;
		let damage = node.query_opt_t::<DamageRoll>("scope() > damage")?;
		Ok(Self {
			kind,
			check,
			area_of_effect,
			damage,
			weapon_kind: None,
			properties: Vec::new(),
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
				}),
				weapon_kind: None,
				properties: Vec::new(),
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
				}),
				weapon_kind: None,
				properties: Vec::new(),
			};
			assert_eq_fromkdl!(Attack, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
