use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt, ValueExt},
	system::dnd5e::{
		data::{character::Character, Ability},
		Value,
	},
	utility::Evaluator,
	GeneralError,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum AttackCheckKind {
	AttackRoll {
		ability: Ability,
		proficient: Value<bool>,
	},
	SavingThrow {
		base: i32,
		dc_ability: Option<Ability>,
		proficient: bool,
		save_ability: Ability,
	},
}

crate::impl_trait_eq!(AttackCheckKind);
impl Evaluator for AttackCheckKind {
	type Context = Character;
	type Item = i32;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::AttackRoll {
				ability,
				proficient,
			} => {
				let proficient = proficient.evaluate(state);
				state.ability_modifier(*ability, Some(proficient.into()))
			}
			Self::SavingThrow {
				base,
				dc_ability,
				proficient,
				save_ability: _,
			} => {
				let ability_bonus = dc_ability
					.as_ref()
					.map(|ability| state.ability_scores().get(*ability).score().modifier())
					.unwrap_or_default();
				let prof_bonus = proficient
					.then(|| state.proficiency_bonus())
					.unwrap_or_default();
				*base + ability_bonus + prof_bonus
			}
		}
	}
}

impl FromKDL for AttackCheckKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"AttackRoll" => {
				let ability = Ability::from_str(node.get_str_req(ctx.consume_idx())?)?;
				let proficient = match (
					node.get_bool_opt("proficient")?,
					node.query("scope() > proficient")?,
				) {
					(None, None) => Value::Fixed(false),
					(Some(prof), None) => Value::Fixed(prof),
					(_, Some(node)) => {
						let mut ctx = ctx.next_node();
						Value::from_kdl(
							node,
							node.entry_req(ctx.consume_idx())?,
							&mut ctx,
							|value| Ok(value.as_bool_req()?),
						)?
					}
				};
				Ok(Self::AttackRoll {
					ability,
					proficient,
				})
			}
			"SavingThrow" => {
				// TODO: The difficulty class should be its own struct (which impls evaluator)
				let (base, dc_ability, proficient) = {
					let node = node.query_req("scope() > difficulty_class")?;
					let mut ctx = ctx.next_node();
					let base = node.get_i64_req(ctx.consume_idx())? as i32;
					let ability = match node.query_str_opt("scope() > ability_bonus", 0)? {
						None => None,
						Some(str) => Some(Ability::from_str(str)?),
					};
					let proficient = node
						.query_bool_opt("scope() > proficiency_bonus", 0)?
						.unwrap_or(false);
					(base, ability, proficient)
				};
				let save_ability =
					Ability::from_str(node.query_str_req("scope() > save_ability", 0)?)?;
				Ok(Self::SavingThrow {
					base,
					dc_ability,
					proficient,
					save_ability,
				})
			}
			name => Err(GeneralError(format!(
				"Invalid attack check {name:?}, expected AttackRoll or SavingThrow"
			))
			.into()),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::dnd5e::{
				data::{evaluator::IsProficientWith, item::weapon, WeaponProficiency},
				NodeRegistry,
			},
		};

		fn from_doc(doc: &str) -> anyhow::Result<AttackCheckKind> {
			let node_reg = NodeRegistry::default_with_eval::<IsProficientWith>();
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > check")?
				.expect("missing check node");
			AttackCheckKind::from_kdl(node, &mut NodeContext::registry(node_reg))
		}

		#[test]
		fn atkroll_simple() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" (Ability)\"Strength\"";
			let expected = AttackCheckKind::AttackRoll {
				ability: Ability::Strength,
				proficient: Value::Fixed(false),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn atkroll_proficient() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" (Ability)\"Strength\" proficient=true";
			let expected = AttackCheckKind::AttackRoll {
				ability: Ability::Strength,
				proficient: Value::Fixed(true),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn atkroll_proficient_eval() -> anyhow::Result<()> {
			let doc = "check \"AttackRoll\" (Ability)\"Strength\" {
				proficient (Evaluator)\"is_proficient_with\" (Weapon)\"Martial\"
			}";
			let expected = AttackCheckKind::AttackRoll {
				ability: Ability::Strength,
				proficient: Value::Evaluated(
					IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial)).into(),
				),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn save_simple() -> anyhow::Result<()> {
			let doc = "check \"SavingThrow\" {
				difficulty_class 8
				save_ability \"CON\"
			}";
			let expected = AttackCheckKind::SavingThrow {
				base: 8,
				dc_ability: None,
				proficient: false,
				save_ability: Ability::Constitution,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn save_dc_ability() -> anyhow::Result<()> {
			let doc = "check \"SavingThrow\" {
				difficulty_class 8 {
					ability_bonus \"WIS\"
				}
				save_ability \"CON\"
			}";
			let expected = AttackCheckKind::SavingThrow {
				base: 8,
				dc_ability: Some(Ability::Wisdom),
				proficient: false,
				save_ability: Ability::Constitution,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn save_dc_proficiency() -> anyhow::Result<()> {
			let doc = "check \"SavingThrow\" {
				difficulty_class 8 {
					proficiency_bonus true
				}
				save_ability \"CON\"
			}";
			let expected = AttackCheckKind::SavingThrow {
				base: 8,
				dc_ability: None,
				proficient: true,
				save_ability: Ability::Constitution,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}

	// TODO: Test AttackCheckKind::Evaluate
}
