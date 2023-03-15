use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{
				character::Character,
				item::{armor, weapon},
				proficiency, Ability, Skill, WeaponProficiency,
			},
			FromKDL,
		},
	},
	utility::{Mutator, Selector, SelectorMeta},
	GeneralError,
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum AddProficiency {
	SavingThrow(Ability),
	Skill(Selector<Skill>, proficiency::Level),
	Language(Selector<String>),
	Armor(armor::Kind),
	Weapon(WeaponProficiency),
	Tool(String),
}

crate::impl_trait_eq!(AddProficiency);
crate::impl_kdl_node!(AddProficiency, "add_proficiency");

impl Mutator for AddProficiency {
	type Target = Character;

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		match &self {
			Self::SavingThrow(ability) => {
				stats.saving_throws_mut().add_proficiency(*ability, source);
			}
			Self::Skill(skill, level) => {
				if let Some(skill) = stats.resolve_selector(skill) {
					stats.skills_mut().add_proficiency(skill, *level, source);
				}
			}
			Self::Language(value) => {
				if let Some(value) = stats.resolve_selector(value) {
					stats
						.other_proficiencies_mut()
						.languages
						.insert(value, source);
				}
			}
			Self::Armor(value) => {
				stats
					.other_proficiencies_mut()
					.armor
					.insert(value.clone(), source);
			}
			Self::Weapon(value) => {
				stats
					.other_proficiencies_mut()
					.weapons
					.insert(value.clone(), source);
			}
			Self::Tool(value) => {
				stats
					.other_proficiencies_mut()
					.tools
					.insert(value.clone(), source);
			}
		}
	}

	fn selector_meta(&self) -> Option<Vec<SelectorMeta>> {
		match self {
			Self::Skill(selector, _) => {
				//Some(vec![selector.as_meta_enum()])
				None
			}
			Self::Language(selector) => {
				//Some(vec![selector.as_meta_str()])
				None
			}
			_ => None,
		}
	}
}

impl FromKDL for AddProficiency {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let entry_idx = value_idx.next();
		let entry = node.entry_req(entry_idx)?;
		let type_id = entry
			.ty()
			.ok_or(GeneralError(format!(
				"Missing proficiency type at value {entry_idx} of {node:?}. \
				Type is required to know what kind of proficiency to add."
			)))?
			.value();
		match type_id {
			"SavingThrow" => Ok(Self::SavingThrow(Ability::from_str(
				node.get_str(entry_idx)?,
			)?)),
			"Skill" => {
				let skill = Selector::from_kdl(node, entry, value_idx, |kdl| {
					Ok(Skill::from_str(kdl.as_string().ok_or(GeneralError(
						format!("Skill selector value {kdl:?} must be a string."),
					))?)?)
				})?;
				let level = match node.get_str_opt("level")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::Full,
				};
				Ok(Self::Skill(skill, level))
			}
			"Language" => {
				let language = Selector::from_kdl(node, entry, value_idx, |kdl| {
					Ok(kdl
						.as_string()
						.ok_or(GeneralError(format!(
							"Skill selector value {kdl:?} must be a string."
						)))?
						.to_owned())
				})?;
				Ok(Self::Language(language))
			}
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

	mod from_kdl {
		use super::*;
		use crate::system::dnd5e::{data::item::weapon, BoxedMutator};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<AddProficiency>(doc)
		}

		#[test]
		fn saving_throw() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (SavingThrow)\"Constitution\"";
			let expected = AddProficiency::SavingThrow(Ability::Constitution);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_specific_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Specific\" \"Insight\"";
			let expected =
				AddProficiency::Skill(Selector::Specific(Skill::Insight), proficiency::Level::Full);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_specific_withlevel() -> anyhow::Result<()> {
			let doc =
				"mutator \"add_proficiency\" (Skill)\"Specific\" \"Religion\" level=\"Double\"";
			let expected = AddProficiency::Skill(
				Selector::Specific(Skill::Religion),
				proficiency::Level::Double,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\" id=\"MutatorSelect\"";
			let expected = AddProficiency::Skill(
				Selector::Any {
					id: Some("MutatorSelect".into()),
				},
				proficiency::Level::Full,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel_noid() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\"";
			let expected =
				AddProficiency::Skill(Selector::Any { id: None }, proficiency::Level::Full);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_any_withlevel() -> anyhow::Result<()> {
			let doc =
				"mutator \"add_proficiency\" (Skill)\"Any\" id=\"MutatorSelect\" level=\"Half\"";
			let expected = AddProficiency::Skill(
				Selector::Any {
					id: Some("MutatorSelect".into()),
				},
				proficiency::Level::Half,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_anyof_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"AnyOf\" id=\"MutatorSelect\" {
				option \"Insight\"
				option \"AnimalHandling\"
			}";
			let expected = AddProficiency::Skill(
				Selector::AnyOf {
					id: Some("MutatorSelect".into()),
					options: vec![Skill::Insight, Skill::AnimalHandling],
				},
				proficiency::Level::Full,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_anyof_withlevel_noid() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"AnyOf\" level=\"Double\" {
				option \"Insight\"
				option \"AnimalHandling\"
			}";
			let expected = AddProficiency::Skill(
				Selector::AnyOf {
					id: None,
					options: vec![Skill::Insight, Skill::AnimalHandling],
				},
				proficiency::Level::Double,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn language_specific() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Language)\"Specific\" \"Gibberish\"";
			let expected = AddProficiency::Language(Selector::Specific("Gibberish".into()));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn language_any() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Language)\"Any\"";
			let expected = AddProficiency::Language(Selector::Any { id: None });
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn language_anyof() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Language)\"AnyOf\" {
				option \"Dwarven\"
				option \"Giant\"
			}";
			let expected = AddProficiency::Language(Selector::AnyOf {
				id: None,
				options: vec!["Dwarven".into(), "Giant".into()],
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn armor() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Medium\"";
			let expected = AddProficiency::Armor(armor::Kind::Medium);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn weapon_simple() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Weapon)\"Simple\"";
			let expected = AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn weapon_martial() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Weapon)\"Martial\"";
			let expected = AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn weapon_class() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Weapon)\"Club\"";
			let expected = AddProficiency::Weapon(WeaponProficiency::Classification("Club".into()));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn tool() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"Dragonchess Set\"";
			let expected = AddProficiency::Tool("Dragonchess Set".into());
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			item::weapon,
			Feature,
		};

		fn character(mutator: AddProficiency) -> Character {
			Character::from(Persistent {
				feats: vec![Feature {
					name: "AddProficiency".into(),
					mutators: vec![mutator.into()],
					..Default::default()
				}
				.into()],
				..Default::default()
			})
		}

		#[test]
		fn saving_throw() {
			let character = character(AddProficiency::SavingThrow(Ability::Dexterity));
			assert_eq!(
				*character
					.saving_throws()
					.get_prof(Ability::Dexterity)
					.value(),
				proficiency::Level::Full,
			);
		}

		#[test]
		fn skill() {
			let character = character(AddProficiency::Skill(
				Selector::Specific(Skill::Arcana),
				proficiency::Level::Double,
			));
			assert_eq!(
				character.skills()[Skill::Arcana].0,
				(
					proficiency::Level::Double,
					vec![("AddProficiency".into(), proficiency::Level::Double)]
				)
					.into(),
			);
		}

		#[test]
		fn language() {
			let character = character(AddProficiency::Language(Selector::Specific(
				"Common".into(),
			)));
			assert_eq!(
				*character.other_proficiencies().languages,
				[("Common".into(), ["AddProficiency".into()].into())].into()
			);
		}

		#[test]
		fn armor() {
			let character = character(AddProficiency::Armor(armor::Kind::Heavy));
			assert_eq!(
				*character.other_proficiencies().armor,
				[(armor::Kind::Heavy, ["AddProficiency".into()].into())].into()
			);
		}

		#[test]
		fn weapon_kind() {
			let character = character(AddProficiency::Weapon(WeaponProficiency::Kind(
				weapon::Kind::Martial,
			)));
			assert_eq!(
				*character.other_proficiencies().weapons,
				[(
					WeaponProficiency::Kind(weapon::Kind::Martial),
					["AddProficiency".into()].into()
				)]
				.into()
			);
		}

		#[test]
		fn weapon_class() {
			let character = character(AddProficiency::Weapon(WeaponProficiency::Classification(
				"Quarterstaff".into(),
			)));
			assert_eq!(
				*character.other_proficiencies().weapons,
				[(
					WeaponProficiency::Classification("Quarterstaff".into()),
					["AddProficiency".into()].into()
				)]
				.into()
			);
		}

		#[test]
		fn tool() {
			let character = character(AddProficiency::Tool("Thieves' Tools".into()));
			assert_eq!(
				*character.other_proficiencies().tools,
				[("Thieves' Tools".into(), ["AddProficiency".into()].into())].into()
			);
		}
	}
}
