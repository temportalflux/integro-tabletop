use crate::{
	kdl_ext::{EntryExt, FromKDL, NodeExt, ValueExt},
	system::dnd5e::data::{
		character::Character, item::weapon, proficiency, Ability, ArmorExtended, Skill,
		WeaponProficiency,
	},
	utility::{Evaluator, NotInList},
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
				*state.saving_throws().get_prof(*ability).value() != proficiency::Level::None
			}
			Self::Skill(skill) => {
				*state.skills().proficiency(*skill).value() != proficiency::Level::None
			}
			Self::Language(language) => {
				state.other_proficiencies().languages.contains_key(language)
			}
			Self::Armor(kind) => {
				state
					.other_proficiencies()
					.armor
					.iter()
					.filter(|((armor, _), _)| armor == kind)
					.count() > 0
			}
			Self::Weapon(proficiency) => state
				.other_proficiencies()
				.weapons
				.contains_key(proficiency),
			Self::Tool(tool) => state.other_proficiencies().tools.contains_key(tool),
		}
	}
}

crate::impl_kdl_node!(IsProficientWith, "is_proficient_with");

impl FromKDL for IsProficientWith {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let entry = node.entry_req(ctx.consume_idx())?;
		match entry.type_req()? {
			"SavingThrow" => Ok(Self::SavingThrow(Ability::from_str(entry.as_str_req()?)?)),
			"Skill" => Ok(Self::Skill(Skill::from_str(entry.as_str_req()?)?)),
			"Language" => Ok(Self::Language(entry.as_str_req()?.to_owned())),
			"Armor" => Ok(Self::Armor(ArmorExtended::from_str(entry.as_str_req()?)?)),
			"Weapon" => Ok(Self::Weapon(match entry.as_str_req()? {
				kind if kind == "Simple" || kind == "Martial" => {
					WeaponProficiency::Kind(weapon::Kind::from_str(kind)?)
				}
				classification => WeaponProficiency::Classification(classification.to_owned()),
			})),
			"Tool" => Ok(Self::Tool(entry.as_str_req()?.to_owned())),
			name => Err(NotInList(
				name.into(),
				vec![
					"SavingThrow",
					"Skill",
					"Language",
					"Armor",
					"Weapon",
					"Tool",
				],
			)
			.into()),
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

	mod from_kdl {
		use super::*;
		use crate::{system::core::NodeRegistry, utility::GenericEvaluator};

		fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, bool>> {
			NodeRegistry::defaulteval_parse_kdl::<IsProficientWith>(doc)
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
		fn armor_kind() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Armor)\"Light\"";
			let expected = IsProficientWith::Armor(ArmorExtended::Kind(armor::Kind::Light));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn armor_shield() -> anyhow::Result<()> {
			let doc = "evaluator \"is_proficient_with\" (Armor)\"Shield\"";
			let expected = IsProficientWith::Armor(ArmorExtended::Shield);
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
			system::dnd5e::{
				data::{item::weapon, Feature},
				mutator::AddProficiency,
			},
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
		fn armor_kind() {
			let empty = Character::from(Persistent::default());
			let with_prof = character_with_profs(vec![AddProficiency::Armor(
				ArmorExtended::Kind(armor::Kind::Light),
				None,
			)]);
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
			let with_prof =
				character_with_profs(vec![AddProficiency::Armor(ArmorExtended::Shield, None)]);
			let eval = IsProficientWith::Armor(ArmorExtended::Shield);
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
			let with_prof = character_with_profs(vec![AddProficiency::Tool(Selector::Specific(
				"Workworking Tools".into(),
			))]);
			let eval = IsProficientWith::Tool("Workworking Tools".into());
			assert_eq!(eval.evaluate(&empty), false);
			assert_eq!(eval.evaluate(&with_prof), true);
		}
	}
}
