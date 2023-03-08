use std::str::FromStr;

use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{
		data::{
			character::Character,
			item::{armor, weapon},
			proficiency, Ability, Skill, WeaponProficiency,
		},
		DnD5e, FromKDL, KDLNode,
	},
	utility::Evaluator,
	GeneralError,
};

#[derive(Clone, PartialEq, Debug)]
pub enum IsProficientWith {
	SavingThrow(Ability),
	Skill(Skill),
	Language(String),
	Armor(armor::Kind),
	Weapon(WeaponProficiency),
	Tool(String),
}

impl crate::utility::TraitEq for IsProficientWith {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for IsProficientWith {
	type Context = Character;
	type Item = bool;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::SavingThrow(ability) => {
				*state.saving_throws().get_prof(*ability).value() != proficiency::Level::None
			}
			Self::Skill(skill) => *state.skills()[*skill].0.value() != proficiency::Level::None,
			Self::Language(language) => {
				state.other_proficiencies().languages.contains_key(language)
			}
			Self::Armor(kind) => state.other_proficiencies().armor.contains_key(kind),
			Self::Weapon(proficiency) => state
				.other_proficiencies()
				.weapons
				.contains_key(proficiency),
			Self::Tool(tool) => state.other_proficiencies().tools.contains_key(tool),
		}
	}
}

impl KDLNode for IsProficientWith {
	fn id() -> &'static str {
		"is_proficient_with"
	}
}

impl FromKDL<DnD5e> for IsProficientWith {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let entry_idx = value_idx.next();
		let entry = node.entry_req(entry_idx)?;
		let type_id = entry
			.ty()
			.ok_or(GeneralError(format!(
				"Missing proficiency type at value {entry_idx} of {node:?}. \
				Type is required to know what kind of proficiency to check."
			)))?
			.value();
		match type_id {
			"SavingThrow" => Ok(Self::SavingThrow(Ability::from_str(
				node.get_str(entry_idx)?,
			)?)),
			"Skill" => Ok(Self::Skill(Skill::from_str(node.get_str(entry_idx)?)?)),
			"Language" => Ok(Self::Language(node.get_str(entry_idx)?.to_owned())),
			"Armor" => Ok(Self::Armor(armor::Kind::from_str(
				node.get_str(entry_idx)?,
			)?)),
			"Weapon" => Ok(Self::Weapon(match node.get_str(entry_idx)? {
				kind if kind == "Simple" || kind == "Martial" => {
					WeaponProficiency::Kind(weapon::Kind::from_str(kind)?)
				}
				classification => WeaponProficiency::Classification(classification.to_owned()),
			})),
			"Tool" => Ok(Self::Tool(node.get_str(entry_idx)?.to_owned())),
			name => Err(GeneralError(format!(
				"Invalid proficiency type {name:?}, expected \
				SavingThrow, Skill, Language, Armor, Weapon, or Tool"
			))
			.into()),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::character::{Character, Persistent};

	mod from_kdl {
		use super::*;
		use crate::{system::dnd5e::DnD5e, utility::GenericEvaluator};

		fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, bool>> {
			DnD5e::defaulteval_parse_kdl::<IsProficientWith>(doc)
		}

		#[test]
		fn saving_throw() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (SavingThrow)\"CHA\"";
			let expected = IsProficientWith::SavingThrow(Ability::Charisma);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Skill)\"Acrobatics\"";
			let expected = IsProficientWith::Skill(Skill::Acrobatics);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn language() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Language)\"Wongle\"";
			let expected = IsProficientWith::Language("Wongle".into());
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn armor() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Armor)\"Light\"";
			let expected = IsProficientWith::Armor(armor::Kind::Light);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn weapon_kind_simple() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Weapon)\"Simple\"";
			let expected = IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn weapon_kind_martial() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Weapon)\"Martial\"";
			let expected = IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn weapon_class() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Weapon)\"Net\"";
			let expected =
				IsProficientWith::Weapon(WeaponProficiency::Classification("Net".into()));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn tool() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Tool)\"Cook's Supplies\"";
			let expected = IsProficientWith::Tool("Cook's Supplies".into());
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::{
			system::dnd5e::data::{item::weapon, mutator::AddProficiency, Feature},
			utility::Selector,
		};

		fn character_with_profs(mutators: Vec<AddProficiency>) -> Character {
			let mut persistent = Persistent::default();
			persistent.feats.push(
				Feature {
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
			let with_prof =
				character_with_profs(vec![AddProficiency::SavingThrow(Ability::Strength)]);
			let eval = IsProficientWith::SavingThrow(Ability::Strength);
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn skill() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Skill(
				Selector::Specific(Skill::SleightOfHand),
				proficiency::Level::Full,
			)]);
			let eval = IsProficientWith::Skill(Skill::SleightOfHand);
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn language() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Language(
				Selector::Specific("Gibberish".into()),
			)]);
			let eval = IsProficientWith::Language("Gibberish".into());
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn armor() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Armor(armor::Kind::Light)]);
			let eval = IsProficientWith::Armor(armor::Kind::Light);
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn weapon_kind() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Weapon(
				WeaponProficiency::Kind(weapon::Kind::Simple),
			)]);
			let eval = IsProficientWith::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple));
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn weapon_class() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Weapon(
				WeaponProficiency::Classification("CrossbowHand".into()),
			)]);
			let eval =
				IsProficientWith::Weapon(WeaponProficiency::Classification("CrossbowHand".into()));
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}

		#[test]
		fn tool() {
			let empty = Character::from(Persistent::default());
			let with_prof =
				character_with_profs(vec![AddProficiency::Tool("Workworking Tools".into())]);
			let eval = IsProficientWith::Tool("Workworking Tools".into());
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}
	}
}
