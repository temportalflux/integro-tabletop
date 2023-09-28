use crate::{
	kdl_ext::{DocumentExt, EntryExt, FromKDL, NodeBuilder, ValueExt},
	system::dnd5e::data::{
		action::AttackQuery,
		character::{spellcasting, Character},
		description,
		roll::EvaluatedRoll,
		Ability, DamageType,
	},
	utility::{Dependencies, Mutator, NotInList},
};

#[derive(Clone, PartialEq, Debug)]
pub enum Bonus {
	AttackRoll {
		bonus: i32,
		query: Vec<AttackQuery>,
	},
	AttackDamage {
		damage: EvaluatedRoll,
		damage_type: Option<DamageType>,
		query: Vec<AttackQuery>,
	},
	AttackAbilityModifier {
		ability: Ability,
		query: Vec<AttackQuery>,
	},
	SpellDamage {
		damage: EvaluatedRoll,
		query: Vec<spellcasting::Filter>,
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
		description::Section {
			title: Some("Bonus".into()),
			content: format!("{self:?}").into(),
			..Default::default()
		}
	}

	fn dependencies(&self) -> Dependencies {
		use crate::kdl_ext::KDLNode;
		let mut deps = Dependencies::from([super::AddFeature::id()]);
		match self {
			Self::AttackRoll { .. } => {}
			Self::AttackDamage { damage, .. } => {
				deps += damage.dependencies();
			}
			Self::AttackAbilityModifier { .. } => {}
			Self::SpellDamage { damage, .. } => {
				deps += damage.dependencies();
			}
			Self::ArmorClass { .. } => {}
		}
		deps
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		match self {
			Self::AttackRoll { bonus, query } => {
				stats.attack_bonuses_mut().add_to_weapon_attacks(
					*bonus,
					query.clone(),
					parent.to_owned(),
				);
			}
			Self::AttackDamage {
				damage,
				damage_type,
				query,
			} => {
				let bonus = damage.evaluate(stats);
				stats.attack_bonuses_mut().add_to_weapon_damage(
					bonus,
					damage_type.clone(),
					query.clone(),
					parent.to_owned(),
				);
			}
			Self::AttackAbilityModifier { ability, query } => {
				stats.attack_bonuses_mut().add_ability_modifier(
					*ability,
					query.clone(),
					parent.to_owned(),
				);
			}
			Self::SpellDamage { damage, query } => {
				let bonus = damage.evaluate(stats);
				stats.attack_bonuses_mut().add_to_spell_damage(
					bonus,
					query.clone(),
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
		let entry = node.next_req()?;
		match (entry.type_opt(), entry.as_str_req()?) {
			(Some("Attack"), "Roll") => {
				let bonus = node.query_i64_req("scope() > bonus", 0)? as i32;
				let query = node.query_all_t::<AttackQuery>("scope() > query")?;
				Ok(Self::AttackRoll { bonus, query })
			}
			(Some("Attack"), "Damage") => {
				let damage = node.query_req_t::<EvaluatedRoll>("scope() > damage")?;
				let damage_type = node.query_str_opt_t::<DamageType>("scope() > damage_type", 0)?;
				let query = node.query_all_t::<AttackQuery>("scope() > query")?;
				Ok(Self::AttackDamage {
					damage,
					damage_type,
					query,
				})
			}
			(Some("Attack"), "AbilityModifier") => {
				let ability = node.next_str_req_t()?;
				let query = node.query_all_t::<AttackQuery>("scope() > query")?;
				Ok(Self::AttackAbilityModifier { ability, query })
			}
			(Some("Spell"), "Damage") => {
				let damage = node.query_req_t::<EvaluatedRoll>("scope() > damage")?;
				let query = node.query_all_t::<spellcasting::Filter>("scope() > query")?;
				Ok(Self::SpellDamage { damage, query })
			}
			(None, "ArmorClass") => {
				let bonus = node.next_i64_req()? as i32;
				let context = node.get_str_opt("context")?.map(str::to_owned);
				Ok(Self::ArmorClass { bonus, context })
			}
			(type_id, name) => Err(NotInList(
				format!(
					"{}{name}",
					type_id.map(|id| format!("({id})")).unwrap_or_default()
				),
				vec![
					"(Attack)Damage",
					"(Attack)Roll",
					"(Attack)AbilityModifier",
					"(Spell)Damage",
					"ArmorClass",
				],
			)
			.into()),
		}
	}
}

impl crate::kdl_ext::AsKdl for Bonus {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::AttackRoll { bonus, query } => {
				node.push_entry_typed("Roll", "Attack");
				node.push_child_entry("bonus", *bonus as i64);
				for query in query {
					node.push_child_t("query", query);
				}
				node
			}
			Self::AttackDamage {
				damage,
				damage_type,
				query,
			} => {
				node.push_entry_typed("Damage", "Attack");
				node.push_child_t("damage", damage);
				if let Some(kind) = damage_type {
					node.push_child_entry("damage_type", kind.to_string());
				}
				for query in query {
					node.push_child_t("query", query);
				}
				node
			}
			Self::AttackAbilityModifier { ability, query } => {
				node.push_entry_typed("AbilityModifier", "Attack");
				node.push_entry_typed(ability.to_string(), "Ability");
				for query in query {
					node.push_child_t("query", query);
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
			Self::SpellDamage { damage, query } => {
				node.push_entry_typed("Damage", "Spell");
				node.push_child_t("damage", damage);
				for query in query {
					node.push_child_t("query", query);
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
			use crate::system::dnd5e::data::item::weapon;

			#[test]
			fn fixed_unrestricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" (Attack)\"Damage\" {
					|    damage 2
					|}
				";
				let data = Bonus::AttackDamage {
					damage: EvaluatedRoll::from(2),
					damage_type: None,
					query: vec![],
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn fixed_restricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" (Attack)\"Damage\" {
					|    damage 5
					|    query {
					|        weapon \"Simple\" \"Martial\"
					|        attack \"Melee\"
					|        ability \"Strength\"
					|        property \"TwoHanded\" false
					|    }
					|}
				";
				let data = Bonus::AttackDamage {
					damage: EvaluatedRoll::from(5),
					damage_type: None,
					query: vec![AttackQuery {
						weapon_kind: weapon::Kind::Martial | weapon::Kind::Simple,
						attack_kind: AttackKind::Melee.into(),
						ability: [Ability::Strength].into(),
						properties: [(weapon::Property::TwoHanded, false)].into(),
						classification: [].into(),
					}],
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn roll_typed() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" (Attack)\"Damage\" {
					|    damage (Roll)\"1d8\"
					|    damage_type \"Radiant\"
					|}
				";
				let data = Bonus::AttackDamage {
					damage: EvaluatedRoll::from((1, Die::D8)),
					damage_type: Some(DamageType::Radiant),
					query: vec![],
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn eval_amt() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" (Attack)\"Damage\" {
					|    damage {
					|        amount (Evaluator)\"get_level\"
					|    }
					|}
				";
				let data = Bonus::AttackDamage {
					damage: EvaluatedRoll {
						amount: Value::Evaluated(GetLevelInt::default().into()),
						die: None,
					},
					damage_type: None,
					query: vec![],
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
					|mutator \"bonus\" (Attack)\"Roll\" {
					|    bonus 2
					|}
				";
				let data = Bonus::AttackRoll {
					bonus: 2,
					query: vec![],
				};
				assert_eq_askdl!(&data, doc);
				assert_eq_fromkdl!(Target, doc, data.into());
				Ok(())
			}

			#[test]
			fn restricted() -> anyhow::Result<()> {
				let doc = "
					|mutator \"bonus\" (Attack)\"Roll\" {
					|    bonus 5
					|    query {
					|        attack \"Ranged\"
					|    }
					|}
				";
				let data = Bonus::AttackRoll {
					bonus: 5,
					query: vec![AttackQuery {
						attack_kind: AttackKind::Ranged.into(),
						..Default::default()
					}],
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

		#[test]
		fn attack_ability_modifier() -> anyhow::Result<()> {
			let doc = "
				|mutator \"bonus\" (Attack)\"AbilityModifier\" (Ability)\"Constitution\" {
				|    query {
				|        class \"Shortsword\"
				|    }
				|}
			";
			let data = Bonus::AttackAbilityModifier {
				ability: Ability::Constitution,
				query: vec![AttackQuery {
					classification: ["Shortsword".into()].into(),
					..Default::default()
				}],
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
