use crate::{
	kdl_ext::{NodeContext, NodeReader},
	system::dnd5e::{data::Ability, Value},
	utility::NotInList,
};
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub enum AttackCheckKind {
	AttackRoll { ability: Ability, proficient: Value<bool>, bonus: i32 },
	SavingThrow { base: i32, dc_ability: Option<Ability>, proficient: bool, save_ability: Ability },
}

crate::impl_trait_eq!(AttackCheckKind);

impl FromKdl<NodeContext> for AttackCheckKind {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"AttackRoll" => {
				let ability = node.next_str_req_t::<Ability>()?;
				let proficient = match (
					node.get_bool_opt("proficient")?,
					node.query_opt_t::<Value<bool>>("scope() > proficient")?,
				) {
					(None, None) => Value::Fixed(false),
					(Some(prof), None) => Value::Fixed(prof),
					(_, Some(value)) => value,
				};
				let bonus = node.get_i64_opt("bonus")?.unwrap_or_default() as i32;
				Ok(Self::AttackRoll { ability, proficient, bonus })
			}
			"SavingThrow" => {
				// TODO: The difficulty class should be its own struct (which impls evaluator)
				let (base, dc_ability, proficient) = {
					let mut node = node.query_req("scope() > difficulty_class")?;
					let base = node.next_i64_req()? as i32;
					let ability = node.query_str_opt_t::<Ability>("scope() > ability_bonus", 0)?;
					let proficient = node.query_bool_opt("scope() > proficiency_bonus", 0)?.unwrap_or(false);
					(base, ability, proficient)
				};
				let save_ability = node.query_str_req_t::<Ability>("scope() > save_ability", 0)?;
				Ok(Self::SavingThrow { base, dc_ability, proficient, save_ability })
			}
			name => Err(NotInList(name.into(), vec!["AttackRoll", "SavingThrow"]).into()),
		}
	}
}

impl AsKdl for AttackCheckKind {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::AttackRoll { ability, proficient, bonus } => {
				node.entry("AttackRoll");
				node.entry_typed("Ability", ability.long_name());
				match proficient {
					Value::Fixed(false) => {}
					Value::Fixed(true) => node.entry(("proficient", true)),
					value => node.child(("proficient", value)),
				}
				if *bonus != 0 {
					node.entry(("bonus", *bonus as i64));
				}
				node
			}
			Self::SavingThrow { base, dc_ability, proficient, save_ability } => {
				node.entry("SavingThrow");
				node.child(("difficulty_class", {
					let mut node = NodeBuilder::default();
					node.entry(*base as i64);
					if let Some(ability) = dc_ability {
						node.child(("ability_bonus", ability.long_name()));
					}
					if *proficient {
						node.child(("proficiency_bonus", true));
					}
					node
				}));
				node.child(("save_ability", save_ability.long_name().as_kdl().with_type("Ability")));
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
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				dnd5e::{
					data::{item::weapon, WeaponProficiency},
					evaluator::IsProficientWith,
				},
				generics,
			},
		};

		static NODE_NAME: &str = "check";

		fn node_ctx() -> NodeContext {
			NodeContext::registry(generics::Registry::default_with_eval::<IsProficientWith>())
		}

		#[test]
		fn atkroll_simple() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" (Ability)\"Strength\"";
			let data =
				AttackCheckKind::AttackRoll { ability: Ability::Strength, proficient: Value::Fixed(false), bonus: 0 };
			assert_eq_fromkdl!(AttackCheckKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn atkroll_proficient() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" (Ability)\"Strength\" proficient=true";
			let data =
				AttackCheckKind::AttackRoll { ability: Ability::Strength, proficient: Value::Fixed(true), bonus: 0 };
			assert_eq_fromkdl!(AttackCheckKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn atkroll_proficient_eval() -> anyhow::Result<()> {
			let doc = "
				|check \"AttackRoll\" (Ability)\"Strength\" {
				|    proficient (Evaluator)\"is_proficient_with\" (Weapon)\"Martial\"
				|}
			";
			let data = AttackCheckKind::AttackRoll {
				ability: Ability::Strength,
				proficient: Value::Evaluated(
					IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial)).into(),
				),
				bonus: 0,
			};
			assert_eq_fromkdl!(AttackCheckKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn save_simple() -> anyhow::Result<()> {
			let doc = "
				|check \"SavingThrow\" {
				|    difficulty_class 8
				|    save_ability (Ability)\"Constitution\"
				|}
			";
			let data = AttackCheckKind::SavingThrow {
				base: 8,
				dc_ability: None,
				proficient: false,
				save_ability: Ability::Constitution,
			};
			assert_eq_fromkdl!(AttackCheckKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn save_dc_ability() -> anyhow::Result<()> {
			let doc = "
				|check \"SavingThrow\" {
				|    difficulty_class 8 {
				|        ability_bonus \"Wisdom\"
				|    }
				|    save_ability (Ability)\"Constitution\"
				|}
			";
			let data = AttackCheckKind::SavingThrow {
				base: 8,
				dc_ability: Some(Ability::Wisdom),
				proficient: false,
				save_ability: Ability::Constitution,
			};
			assert_eq_fromkdl!(AttackCheckKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn save_dc_proficiency() -> anyhow::Result<()> {
			let doc = "
				|check \"SavingThrow\" {
				|    difficulty_class 8 {
				|        proficiency_bonus true
				|    }
				|    save_ability (Ability)\"Constitution\"
				|}
			";
			let data = AttackCheckKind::SavingThrow {
				base: 8,
				dc_ability: None,
				proficient: true,
				save_ability: Ability::Constitution,
			};
			assert_eq_fromkdl!(AttackCheckKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}

	// TODO: Test AttackCheckKind::Evaluate
}
