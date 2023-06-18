use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{character::Character, Ability},
	utility::{Dependencies, Evaluator},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct GetAbilityModifier(pub Ability);

crate::impl_trait_eq!(GetAbilityModifier);
crate::impl_kdl_node!(GetAbilityModifier, "get_ability_modifier");

impl Evaluator for GetAbilityModifier {
	type Context = Character;
	type Item = i32;

	fn description(&self) -> Option<String> {
		Some(format!("your {} modifier", self.0.long_name()))
	}

	fn dependencies(&self) -> Dependencies {
		["ability_score_finalize"].into()
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let value = state.ability_modifier(self.0, None);
		value
	}
}

impl FromKDL for GetAbilityModifier {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(Ability::from_str(
			node.get_str_req(ctx.consume_idx())?,
		)?))
	}
}

impl AsKdl for GetAbilityModifier {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default().with_entry_typed(self.0.long_name(), "Ability")
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::character::Persistent;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::evaluator::test::test_utils};

		test_utils!(GetAbilityModifier);

		#[test]
		fn ability_str() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Strength\"";
			let data = GetAbilityModifier(Ability::Strength);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn ability_dex() -> anyhow::Result<()> {
			let doc_in = "evaluator \"get_ability_modifier\" (Ability)\"DEX\"";
			let doc_out = "evaluator \"get_ability_modifier\" (Ability)\"Dexterity\"";
			let data = GetAbilityModifier(Ability::Dexterity);
			assert_eq_askdl!(&data, doc_out);
			assert_eq_fromkdl!(Target, doc_in, data.into());
			Ok(())
		}

		#[test]
		fn ability_con() -> anyhow::Result<()> {
			let doc_in = "evaluator \"get_ability_modifier\" (Ability)\"con\"";
			let doc_out = "evaluator \"get_ability_modifier\" (Ability)\"Constitution\"";
			let data = GetAbilityModifier(Ability::Constitution);
			assert_eq_askdl!(&data, doc_out);
			assert_eq_fromkdl!(Target, doc_in, data.into());
			Ok(())
		}

		#[test]
		fn ability_int() -> anyhow::Result<()> {
			let doc_in = "evaluator \"get_ability_modifier\" (Ability)\"Int\"";
			let doc_out = "evaluator \"get_ability_modifier\" (Ability)\"Intelligence\"";
			let data = GetAbilityModifier(Ability::Intelligence);
			assert_eq_askdl!(&data, doc_out);
			assert_eq_fromkdl!(Target, doc_in, data.into());
			Ok(())
		}

		#[test]
		fn ability_wis() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Wisdom\"";
			let data = GetAbilityModifier(Ability::Wisdom);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn ability_cha() -> anyhow::Result<()> {
			let doc = "evaluator \"get_ability_modifier\" (Ability)\"Charisma\"";
			let data = GetAbilityModifier(Ability::Charisma);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod evaluate {
		use super::*;

		fn character(scores: &[(Ability, u32)]) -> Character {
			let mut persistent = Persistent::default();
			for (ability, score) in scores {
				persistent.ability_scores[*ability] = *score;
			}
			Character::from(persistent)
		}

		#[test]
		fn base_score_default() {
			let character = character(&[
				(Ability::Strength, 10),
				(Ability::Dexterity, 10),
				(Ability::Constitution, 10),
				(Ability::Intelligence, 10),
				(Ability::Wisdom, 10),
				(Ability::Charisma, 10),
			]);
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
