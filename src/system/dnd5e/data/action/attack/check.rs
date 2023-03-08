use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::dnd5e::{
		data::{character::Character, Ability},
		DnD5e, FromKDL, Value,
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

impl crate::utility::TraitEq for AttackCheckKind {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

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
					.map(|ability| state.ability_score(*ability).0.modifier())
					.unwrap_or_default();
				let prof_bonus = proficient
					.then(|| state.proficiency_bonus())
					.unwrap_or_default();
				*base + ability_bonus + prof_bonus
			}
		}
	}
}

impl FromKDL<DnD5e> for AttackCheckKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"AttackRoll" => {
				let ability = Ability::from_str(node.get_str(value_idx.next())?)?;
				let proficient = match (node.get_bool_opt("proficient")?, node.query("proficient")?)
				{
					(None, None) => Value::Fixed(false),
					(Some(prof), None) => Value::Fixed(prof),
					(_, Some(node)) => {
						let mut value_idx = ValueIdx::default();
						Value::from_kdl(
							node,
							node.entry_req(value_idx.next())?,
							&mut value_idx,
							system,
							|value| Ok(value.as_bool()),
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
					let node = node.query_req("difficulty_class")?;
					let mut value_idx = ValueIdx::default();
					let base = node.get_i64(value_idx.next())? as i32;
					let ability = match node.query_str_opt("ability_bonus", 0)? {
						None => None,
						Some(str) => Some(Ability::from_str(str)?),
					};
					let proficient = node
						.query_bool_opt("proficiency_bonus", 0)?
						.unwrap_or(false);
					(base, ability, proficient)
				};
				let save_ability = Ability::from_str(node.query_str("save_ability", 0)?)?;
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
