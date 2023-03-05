use std::str::FromStr;

use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{
		data::{character::Character, Ability},
		DnD5e, FromKDL, KDLNode,
	},
	utility::{Dependencies, Evaluator},
	GeneralError,
};

#[derive(Clone, PartialEq, Debug)]
pub struct GetAbilityModifier(pub Ability);

impl crate::utility::TraitEq for GetAbilityModifier {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for GetAbilityModifier {
	type Context = Character;
	type Item = i32;

	fn dependencies(&self) -> Dependencies {
		["add_ability_score"].into()
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let value = state.ability_modifier(self.0, None);
		value
	}
}

impl KDLNode for GetAbilityModifier {
	fn id() -> &'static str {
		"get_ability_modifier"
	}
}

impl FromKDL<DnD5e> for GetAbilityModifier {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let ability_str = node.get_str(value_idx.next())?;
		let ability = Ability::from_str(ability_str)
			.map_err(|_| GeneralError(format!("Invalid ability name {ability_str:?}")))?;
		Ok(Self(ability))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		system::dnd5e::{data::character::Persistent, DnD5e},
		utility::GenericEvaluator,
	};

	fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, i32>> {
		DnD5e::defaulteval_parse_kdl::<GetAbilityModifier>(doc)
	}

	mod from_kdl {
		use super::*;
		use crate::system::dnd5e::data::evaluator::GetAbilityModifier;

		#[test]
		fn ability_str() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Strength\"";
			let expected = GetAbilityModifier(Ability::Strength);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn ability_dex() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"DEX\"";
			let expected = GetAbilityModifier(Ability::Dexterity);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn ability_con() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"con\"";
			let expected = GetAbilityModifier(Ability::Constitution);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn ability_int() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Int\"";
			let expected = GetAbilityModifier(Ability::Intelligence);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn ability_wis() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Wisdom\"";
			let expected = GetAbilityModifier(Ability::Wisdom);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn ability_cha() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Charisma\"";
			let expected = GetAbilityModifier(Ability::Charisma);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod evaluate {
		use super::*;

		fn character(scores: &[(Ability, u32)]) -> Character {
			let mut persistent = Persistent::default();
			for (ability, score) in scores {
				*persistent.ability_scores[*ability] = *score;
			}
			Character::from(persistent)
		}

		#[test]
		fn base_score_default() {
			let character = character(&[]);
			let str = GetAbilityModifier(Ability::Strength);
			let dex = GetAbilityModifier(Ability::Dexterity);
			let con = GetAbilityModifier(Ability::Constitution);
			let int = GetAbilityModifier(Ability::Intelligence);
			let wis = GetAbilityModifier(Ability::Wisdom);
			let cha = GetAbilityModifier(Ability::Charisma);
			assert_eq!(str.evaluate(&character), 0);
			assert_eq!(dex.evaluate(&character), 0);
			assert_eq!(con.evaluate(&character), 0);
			assert_eq!(int.evaluate(&character), 0);
			assert_eq!(wis.evaluate(&character), 0);
			assert_eq!(cha.evaluate(&character), 0);
		}

		#[test]
		fn base_score_positive() {
			let character = character(&[(Ability::Intelligence, 15)]);
			let int = GetAbilityModifier(Ability::Intelligence);
			assert_eq!(int.evaluate(&character), 2);
		}

		#[test]
		fn base_score_negative() {
			let character = character(&[(Ability::Dexterity, 7)]);
			let dex = GetAbilityModifier(Ability::Dexterity);
			assert_eq!(dex.evaluate(&character), -2);
		}
	}
}
