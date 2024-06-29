use super::super::{AreaOfEffect, DamageRoll};
use crate::{
	kdl_ext::NodeContext,
	system::dnd5e::data::{character::Character, item::weapon, Ability},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::collections::HashSet;

mod check;
pub use check::*;
mod kind;
pub use kind::*;
mod range;
pub use range::*;
mod query;
pub use query::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Attack {
	pub kind: Option<AttackKindValue>,
	pub check: AttackCheckKind,
	pub area_of_effect: Option<AreaOfEffect>,
	pub damage: Option<DamageRoll>,
	pub weapon_kind: Option<weapon::Kind>,
	pub classification: Option<String>,
	pub properties: Vec<weapon::Property>,
}

impl FromKdl<NodeContext> for Attack {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = node.query_opt_t::<AttackKindValue>("scope() > kind")?;
		let check = node.query_req_t::<AttackCheckKind>("scope() > check")?;
		let area_of_effect = node.query_opt_t::<AreaOfEffect>("scope() > area_of_effect")?;
		let damage = node.query_opt_t::<DamageRoll>("scope() > damage")?;
		let classification = node.get_str_opt("class")?.map(str::to_owned);
		Ok(Self { kind, check, area_of_effect, damage, weapon_kind: None, classification, properties: Vec::new() })
	}
}

impl AsKdl for Attack {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(classification) = &self.classification {
			node.entry(("class", classification.clone()));
		}
		node.child(("kind", &self.kind));
		node.child(("check", &self.check));
		node.child(("area_of_effect", &self.area_of_effect));
		node.child(("damage", &self.damage));
		node
	}
}

impl Attack {
	fn all_ability_options(&self, primary: Ability, state: &Character) -> HashSet<Ability> {
		let mut abilities = HashSet::from([primary]);
		if self.properties.contains(&weapon::Property::Finesse) {
			abilities.extend([Ability::Strength, Ability::Dexterity]);
		}
		abilities.extend(state.attack_bonuses().get_attack_ability_variants(self));
		return abilities;
	}

	pub fn best_ability_modifier(&self, primary: Ability, state: &Character) -> (Ability, i32) {
		let abilities = self.all_ability_options(primary, state).into_iter();
		let abilities = abilities.map(|ability| {
			let modifier = state.ability_modifier(ability, None);
			(ability, modifier)
		});
		let option = abilities.max_by_key(|(_, modifier)| *modifier);
		return option.expect("there is always at least one ability option");
	}

	pub fn evaluate_bonuses(&self, state: &Character) -> (Option<Ability>, i32, i32) {
		match &self.check {
			AttackCheckKind::AttackRoll { ability, proficient } => {
				let (ability, modifier) = self.best_ability_modifier(*ability, state);
				let prof_bonus = proficient.evaluate(state).then_some(state.proficiency_bonus()).unwrap_or_default();
				let atk_bonus = modifier + prof_bonus;
				let dmg_bonus = modifier;
				(Some(ability), atk_bonus, dmg_bonus)
			}
			AttackCheckKind::SavingThrow { base, dc_ability, proficient, save_ability: _ } => {
				let ability_bonus = dc_ability
					.as_ref()
					.map(|ability| state.ability_scores().get(*ability).score().modifier())
					.unwrap_or_default();
				let prof_bonus = proficient.then(|| state.proficiency_bonus()).unwrap_or_default();
				let atk_bonus = *base + ability_bonus + prof_bonus;
				(None, atk_bonus, 0)
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
				classification: None,
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
				kind: Some(AttackKindValue::Ranged { short_dist: 20, long_dist: 60 }),
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
				classification: None,
				properties: Vec::new(),
			};
			assert_eq_fromkdl!(Attack, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
