use crate::{
	kdl_ext::{DocumentExt, DocumentQueryExt, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{
		character::Character, description, item::weapon, roll::EvaluatedRoll, DamageType,
	},
	utility::{Dependencies, Mutator, NotInList},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum Bonus {
	WeaponDamage {
		damage: EvaluatedRoll,
		damage_type: Option<DamageType>,
		restriction: Option<weapon::Restriction>,
	},
	WeaponAttackRoll {
		bonus: i32,
		restriction: Option<weapon::Restriction>,
	},
	ArmorClass {
		bonus: i32,
		context: Option<String>,
	},
}

crate::impl_trait_eq!(Bonus);
crate::impl_kdl_node!(Bonus, "bonus");

impl Mutator for Bonus {
	type Target = Character;

	fn description(&self, _state: Option<&Self::Target>) -> description::Section {
		// TODO: Bonus description
		description::Section::default()
	}

	fn dependencies(&self) -> Dependencies {
		use crate::kdl_ext::KDLNode;
		let mut deps = Dependencies::from([super::AddFeature::id()]);
		match self {
			Self::WeaponDamage { damage, .. } => {
				deps = deps.join(damage.dependencies());
			}
			Self::WeaponAttackRoll { .. } => {}
			Self::ArmorClass { .. } => {}
		}
		deps
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		match self {
			Self::WeaponDamage {
				damage,
				damage_type,
				restriction,
			} => {
				let bonus = damage.evaluate(stats);
				stats.attack_bonuses_mut().add_to_weapon_damage(
					bonus,
					damage_type.clone(),
					restriction.clone(),
					parent.to_owned(),
				);
			}
			Self::WeaponAttackRoll { bonus, restriction } => {
				stats.attack_bonuses_mut().add_to_weapon_attacks(
					*bonus,
					restriction.clone(),
					parent.to_owned(),
				);
			}
			Self::ArmorClass { bonus, context } => {
				stats
					.armor_class_mut()
					.push_bonus(*bonus, context.clone(), parent.to_owned());
			}
		}
	}
}

impl FromKDL for Bonus {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"WeaponDamage" => {
				let damage =
					EvaluatedRoll::from_kdl(&mut node.query_req("scope() > damage")?)?;
				let damage_type = match node.query_str_opt("scope() > damage_type", 0)? {
					None => None,
					Some(str) => Some(DamageType::from_str(str)?),
				};
				let restriction = match node.query_opt("scope() > restriction")? {
					None => None,
					Some(mut node) => Some(weapon::Restriction::from_kdl(&mut node)?),
				};
				Ok(Self::WeaponDamage {
					damage,
					damage_type,
					restriction,
				})
			}
			"WeaponAttackRoll" => {
				let bonus = node.query_i64_req("scope() > bonus", 0)? as i32;
				let restriction = match node.query_opt("scope() > restriction")? {
					None => None,
					Some(mut node) => Some(weapon::Restriction::from_kdl(&mut node)?),
				};
				Ok(Self::WeaponAttackRoll { bonus, restriction })
			}
			"ArmorClass" => {
				let bonus = node.next_i64_req()? as i32;
				let context = node.get_str_opt("context")?.map(str::to_owned);
				Ok(Self::ArmorClass { bonus, context })
			}
			key => Err(NotInList(
				key.into(),
				vec!["WeaponDamage", "WeaponAttackRoll", "ArmorClass"],
			)
			.into()),
		}
	}
}

impl crate::kdl_ext::AsKdl for Bonus {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::WeaponDamage {
				damage,
				damage_type,
				restriction,
			} => {
				node.push_entry("WeaponDamage");
				node.push_child_t("damage", damage);
				if let Some(kind) = damage_type {
					node.push_child_entry("damage_type", kind.to_string());
				}
				if let Some(restriction) = restriction {
					node.push_child_t("restriction", restriction);
				}
				node
			}
			Self::WeaponAttackRoll { bonus, restriction } => {
				node.push_entry("WeaponAttackRoll");
				node.push_child_entry("bonus", *bonus as i64);
				if let Some(restriction) = restriction {
					node.push_child_t("restriction", restriction);
				}
				node
			}
			Self::ArmorClass { bonus, context } => {
				node.push_entry("ArmorClass");
				node.push_entry(*bonus as i64);
				if let Some(context) = context {
					node.push_entry(("context", context.clone()));
				}
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::{assert_eq_askdl, assert_eq_fromkdl, from_doc, raw_doc},
			system::{
				core::NodeRegistry,
				dnd5e::{
					data::{action::AttackKind, roll::Die, Ability},
					evaluator::GetLevelInt,
					mutator::test::test_utils,
					Value,
				},
			},
		};

		test_utils!(Bonus, node_reg());

		fn node_reg() -> NodeRegistry {
			let mut node_reg = NodeRegistry::default();
			node_reg.register_mutator::<Bonus>();
			node_reg.register_evaluator::<GetLevelInt>();
			node_reg
		}

		mod weapon_damage {
			use super::*;

			#[test]
			fn fixed_unrestricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" \"WeaponDamage\" {
					|    damage 2
					|}
				";
				let data = Bonus::WeaponDamage {
					damage: EvaluatedRoll::from(2),
					damage_type: None,
					restriction: None,
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn fixed_restricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" \"WeaponDamage\" {
					|    damage 5
					|    restriction {
					|        weapon \"Simple\" \"Martial\"
					|        attack \"Melee\"
					|        ability \"Strength\"
					|        property \"TwoHanded\" false
					|    }
					|}
				";
				let data = Bonus::WeaponDamage {
					damage: EvaluatedRoll::from(5),
					damage_type: None,
					restriction: Some(weapon::Restriction {
						weapon_kind: weapon::Kind::Martial | weapon::Kind::Simple,
						attack_kind: AttackKind::Melee.into(),
						ability: [Ability::Strength].into(),
						properties: [(weapon::Property::TwoHanded, false)].into(),
					}),
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn roll_typed() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" \"WeaponDamage\" {
					|    damage (Roll)\"1d8\"
					|    damage_type \"Radiant\"
					|}
				";
				let data = Bonus::WeaponDamage {
					damage: EvaluatedRoll::from((1, Die::D8)),
					damage_type: Some(DamageType::Radiant),
					restriction: None,
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn eval_amt() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" \"WeaponDamage\" {
					|    damage {
					|        amount (Evaluator)\"get_level\"
					|    }
					|}
				";
				let data = Bonus::WeaponDamage {
					damage: EvaluatedRoll {
						amount: Value::Evaluated(GetLevelInt::default().into()),
						die: None,
					},
					damage_type: None,
					restriction: None,
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}
		}

		mod weapon_attack_roll {
			use super::*;

			#[test]
			fn unrestricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" \"WeaponAttackRoll\" {
					|    bonus 2
					|}
				";
				let data = Bonus::WeaponAttackRoll {
					bonus: 2,
					restriction: None,
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn restricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" \"WeaponAttackRoll\" {
					|    bonus 5
					|    restriction {
					|        attack \"Ranged\"
					|    }
					|}
				";
				let data = Bonus::WeaponAttackRoll {
					bonus: 5,
					restriction: Some(weapon::Restriction {
						attack_kind: AttackKind::Ranged.into(),
						..Default::default()
					}),
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}
		}

		mod armor_class {
			use super::*;

			#[test]
			fn no_context() -> anyhow::Result<()> {
				let doc = "mutator \"bonus\" \"ArmorClass\" 1";
				let data = Bonus::ArmorClass {
					bonus: 1,
					context: None,
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn with_context() -> anyhow::Result<()> {
				let doc = "mutator \"bonus\" \"ArmorClass\" 2 context=\"against ranged attacks\"";
				let data = Bonus::ArmorClass {
					bonus: 2,
					context: Some("against ranged attacks".into()),
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}
		}
	}
}
