use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::data::{
			character::Character, item::weapon, proficiency, Ability, ArmorExtended, Skill, WeaponProficiency,
		},
		Evaluator,
	},
	utility::NotInList,
};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum IsProficientWith {
	SavingThrow(Ability),
	Skill(Skill),
	Language(String),
	Armor(ArmorExtended),
	Weapon(WeaponProficiency),
	Tool(String),
}

crate::impl_trait_eq!(IsProficientWith);
impl Evaluator for IsProficientWith {
	type Context = Character;
	type Item = bool;

	fn description(&self) -> Option<String> {
		None
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::SavingThrow(ability) => {
				state.saving_throws()[*ability].proficiencies().value() != proficiency::Level::None
			}
			Self::Skill(skill) => state.skills()[*skill].proficiencies().value() != proficiency::Level::None,
			Self::Language(language) => state.other_proficiencies().languages.contains_key(language),
			Self::Armor(kind) => {
				state.other_proficiencies().armor.iter().filter(|((armor, _), _)| armor == kind).count() > 0
			}
			Self::Weapon(proficiency) => state.other_proficiencies().weapons.contains_key(proficiency),
			Self::Tool(tool) => state.other_proficiencies().tools.contains_key(tool),
		}
	}
}

kdlize::impl_kdl_node!(IsProficientWith, "is_proficient_with");

impl FromKdl<NodeContext> for IsProficientWith {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		match entry.type_req()? {
			"SavingThrow" => Ok(Self::SavingThrow(Ability::from_str(entry.as_str_req()?)?)),
			"Skill" => Ok(Self::Skill(Skill::from_str(entry.as_str_req()?)?)),
			"Language" => Ok(Self::Language(entry.as_str_req()?.to_owned())),
			"Armor" => Ok(Self::Armor(ArmorExtended::from_str(entry.as_str_req()?)?)),
			"Weapon" => Ok(Self::Weapon(match entry.as_str_req()? {
				kind if kind == "Simple" || kind == "Martial" => WeaponProficiency::Kind(weapon::Kind::from_str(kind)?),
				classification => WeaponProficiency::Classification(classification.to_owned()),
			})),
			"Tool" => Ok(Self::Tool(entry.as_str_req()?.to_owned())),
			name => {
				Err(NotInList(name.into(), vec!["SavingThrow", "Skill", "Language", "Armor", "Weapon", "Tool"]).into())
			}
		}
	}
}

impl AsKdl for IsProficientWith {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::SavingThrow(ability) => node.with_entry_typed(ability.long_name(), "SavingThrow"),
			Self::Skill(skill) => node.with_entry_typed(skill.to_string(), "Skill"),
			Self::Language(lang_name) => node.with_entry_typed(lang_name.clone(), "Language"),
			Self::Armor(armor_ext) => node.with_entry_typed(armor_ext.to_string(), "Armor"),
			Self::Weapon(weapon_prof) => node.with_entry_typed(weapon_prof.to_string(), "Weapon"),
			Self::Tool(tool_name) => node.with_entry_typed(tool_name.clone(), "Tool"),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::{
		character::{Character, Persistent},
		item::armor,
	};

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::evaluator::test::test_utils};

		test_utils!(IsProficientWith);

		#[test]
		fn saving_throw() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (SavingThrow)\"Charisma\"";
			let data = IsProficientWith::SavingThrow(Ability::Charisma);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Skill)\"Acrobatics\"";
			let data = IsProficientWith::Skill(Skill::Acrobatics);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn language() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Language)\"Wongle\"";
			let data = IsProficientWith::Language("Wongle".into());
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn armor_kind() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Armor)\"Light\"";
			let data = IsProficientWith::Armor(ArmorExtended::Kind(armor::Kind::Light));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn armor_shield() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Armor)\"Shield\"";
			let data = IsProficientWith::Armor(ArmorExtended::Shield);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn weapon_kind_simple() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Weapon)\"Simple\"";
			let data = IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn weapon_kind_martial() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Weapon)\"Martial\"";
			let data = IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn weapon_class() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Weapon)\"Net\"";
			let data = IsProficientWith::Weapon(WeaponProficiency::Classification("Net".into()));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn tool() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Tool)\"Cook's Supplies\"";
			let data = IsProficientWith::Tool("Cook's Supplies".into());
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::{
			system::dnd5e::{
				data::{item::weapon, Bundle},
				mutator::AddProficiency,
			},
			utility::selector,
		};

		fn character_with_profs(mutators: Vec<AddProficiency>) -> Character {
			let mut persistent = Persistent::default();
			persistent.bundles.push(
				Bundle {
					name: "CustomFeat".into(),
					mutators: mutators.into_iter().map(|v| v.into()).collect(),
					..Default::default()
				}
				.into(),
			);
			Character::from(persistent)
		}

		#[test]
		fn saving_throw() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::SavingThrow(Ability::Strength)]);
			let eval = IsProficientWith::SavingThrow(Ability::Strength);
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn skill() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Skill {
				skill: selector::Value::Specific(Skill::SleightOfHand),
				minimum_level: proficiency::Level::None,
				level: proficiency::Level::Full,
			}]);
			let eval = IsProficientWith::Skill(Skill::SleightOfHand);
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn language() {
			let empty = Character::from(Persistent::default());
			let with_prof =
				character_with_profs(vec![AddProficiency::Language(selector::Value::Specific("Gibberish".into()))]);
			let eval = IsProficientWith::Language("Gibberish".into());
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn armor_kind() {
			let empty = Character::from(Persistent::default());
			let with_prof =
				character_with_profs(vec![AddProficiency::Armor(ArmorExtended::Kind(armor::Kind::Light), None)]);
			let with_prof_ctx = character_with_profs(vec![AddProficiency::Armor(
				ArmorExtended::Kind(armor::Kind::Light),
				Some("nonmetal".into()),
			)]);
			let eval = IsProficientWith::Armor(ArmorExtended::Kind(armor::Kind::Light));
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
			assert_eq!(eval.evaluate(&with_prof_ctx), true);
		}

		#[test]
		fn armor_shield() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Armor(ArmorExtended::Shield, None)]);
			let eval = IsProficientWith::Armor(ArmorExtended::Shield);
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn weapon_kind() {
			let empty = Character::from(Persistent::default());
			let with_prof =
				character_with_profs(vec![AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple))]);
			let eval = IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple));
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn weapon_class() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Weapon(WeaponProficiency::Classification(
				"CrossbowHand".into(),
			))]);
			let eval = IsProficientWith::Weapon(WeaponProficiency::Classification("CrossbowHand".into()));
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn tool() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Tool {
				tool: selector::Value::Specific("Workworking Tools".into()),
				level: proficiency::Level::Full,
			}]);
			let eval = IsProficientWith::Tool("Workworking Tools".into());
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}
	}
}
