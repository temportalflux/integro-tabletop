use crate::{
	kdl_ext::{EntryExt, FromKDL, NodeExt, ValueExt},
	system::dnd5e::data::{
		character::Character, item::weapon, proficiency, Ability, ArmorExtended, Skill,
		WeaponProficiency,
	},
	utility::{Mutator, NotInList, Selector, SelectorMeta, SelectorMetaVec},
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum AddProficiency {
	SavingThrow(Ability),
	Skill(Selector<Skill>, proficiency::Level),
	Language(Selector<String>),
	Armor(ArmorExtended, Option<String>),
	Weapon(WeaponProficiency),
	Tool(Selector<String>),
}

crate::impl_trait_eq!(AddProficiency);
crate::impl_kdl_node!(AddProficiency, "add_proficiency");

impl Mutator for AddProficiency {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some("Proficiency".into())
	}

	fn description(&self) -> Option<String> {
		match self {
			Self::SavingThrow(ability) => Some(format!(
				"You are proficient with {} saving throws.",
				ability.long_name()
			)),
			Self::Skill(Selector::Specific(skill), level) => Some(format!(
				"You are {} with {} checks.",
				level.as_display_name().to_lowercase(),
				skill.display_name()
			)),
			Self::Skill(Selector::Any { .. }, level) => Some(format!(
				"You are {} with one skill of your choice.",
				level.as_display_name().to_lowercase()
			)),
			Self::Skill(Selector::AnyOf { options, .. }, level) => Some(format!(
				"You are {} with one skill of: {}.",
				level.as_display_name().to_lowercase(),
				options
					.iter()
					.map(Skill::display_name)
					.collect::<Vec<_>>()
					.join(", ")
			)),
			Self::Language(Selector::Specific(lang)) => {
				Some(format!("You can speak, read, and write {lang}."))
			}
			Self::Language(Selector::Any { .. }) => Some(format!(
				"You can speak, read, and write one language of your choice."
			)),
			Self::Language(Selector::AnyOf { options, .. }) => Some(format!(
				"You can speak, read, and write one language of: {}.",
				options.join(", ")
			)),
			Self::Armor(kind, context) => {
				let ctx = context
					.as_ref()
					.map(|s| format!(" ({s})"))
					.unwrap_or_default();
				Some(match kind {
					ArmorExtended::Kind(kind) => format!(
						"You are proficient with {} armor{ctx}.",
						kind.to_string().to_lowercase()
					),
					ArmorExtended::Shield => format!("You are proficient with shields{ctx}."),
				})
			}
			Self::Weapon(WeaponProficiency::Kind(kind)) => Some(format!(
				"You are proficient with {} weapons.",
				kind.to_string().to_lowercase()
			)),
			Self::Weapon(WeaponProficiency::Classification(kind)) => {
				Some(format!("You are proficient with {kind} weapon-types."))
			}
			Self::Tool(Selector::Specific(tool)) => {
				Some(format!("You are proficient with {tool}."))
			}
			Self::Tool(Selector::Any { .. }) => {
				Some(format!("You are proficient with one tool of your choice."))
			}
			Self::Tool(Selector::AnyOf { options, .. }) => Some(format!(
				"You are proficient with one tool of: {}.",
				options.join(", ")
			)),
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		match self {
			Self::Skill(selector, _) => selector.set_data_path(parent),
			Self::Language(selector) => selector.set_data_path(parent),
			Self::Tool(selector) => selector.set_data_path(parent),
			_ => {}
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		match &self {
			Self::SavingThrow(ability) => {
				stats
					.saving_throws_mut()
					.add_proficiency(*ability, parent.to_owned());
			}
			Self::Skill(skill, level) => {
				if let Some(skill) = stats.resolve_selector(skill) {
					stats
						.skills_mut()
						.add_proficiency(skill, *level, parent.to_owned());
				}
			}
			Self::Language(value) => {
				if let Some(value) = stats.resolve_selector(value) {
					stats
						.other_proficiencies_mut()
						.languages
						.insert(value, parent.to_owned());
				}
			}
			Self::Armor(value, context) => {
				stats
					.other_proficiencies_mut()
					.armor
					.insert((value.clone(), context.clone()), parent.to_owned());
			}
			Self::Weapon(value) => {
				stats
					.other_proficiencies_mut()
					.weapons
					.insert(value.clone(), parent.to_owned());
			}
			Self::Tool(value) => {
				if let Some(value) = stats.resolve_selector(value) {
					stats
						.other_proficiencies_mut()
						.tools
						.insert(value, parent.to_owned());
				}
			}
		}
	}

	fn selector_meta(&self) -> Option<Vec<SelectorMeta>> {
		match self {
			Self::Skill(selector, _) => SelectorMetaVec::default()
				.with_enum("Skill", selector)
				.to_vec(),
			Self::Language(selector) => SelectorMetaVec::default()
				.with_str("Language", selector)
				.to_vec(),
			Self::Tool(selector) => SelectorMetaVec::default()
				.with_str("Tool", selector)
				.to_vec(),
			_ => None,
		}
	}
}

impl FromKDL for AddProficiency {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let entry = node.entry_req(ctx.consume_idx())?;
		match entry.type_req()? {
			"SavingThrow" => Ok(Self::SavingThrow(Ability::from_str(entry.as_str_req()?)?)),
			"Skill" => {
				let skill = Selector::from_kdl(node, entry, ctx, |kdl| {
					Ok(Skill::from_str(kdl.as_str_req()?)?)
				})?;
				let level = match node.get_str_opt("level")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::Full,
				};
				Ok(Self::Skill(skill, level))
			}
			"Language" => {
				let language =
					Selector::from_kdl(node, entry, ctx, |kdl| Ok(kdl.as_str_req()?.to_owned()))?;
				Ok(Self::Language(language))
			}
			"Armor" => {
				let kind = ArmorExtended::from_str(entry.as_str_req()?)?;
				let context = node.get_str_opt(ctx.consume_idx())?.map(str::to_owned);
				Ok(Self::Armor(kind, context))
			}
			"Weapon" => Ok(Self::Weapon(match entry.as_str_req()? {
				kind if kind == "Simple" || kind == "Martial" => {
					WeaponProficiency::Kind(weapon::Kind::from_str(kind)?)
				}
				classification => WeaponProficiency::Classification(classification.to_owned()),
			})),
			"Tool" => {
				let tool =
					Selector::from_kdl(node, entry, ctx, |kdl| Ok(kdl.as_str_req()?.to_owned()))?;
				Ok(Self::Tool(tool))
			}
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
	use crate::system::dnd5e::data::item::{armor, weapon};

	mod from_kdl {
		use super::*;
		use crate::system::{core::NodeRegistry, dnd5e::BoxedMutator};

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
					id: Some("MutatorSelect").into(),
					cannot_match: Default::default(),
				},
				proficiency::Level::Full,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel_noid() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\"";
			let expected = AddProficiency::Skill(
				Selector::Any {
					id: Default::default(),
					cannot_match: Default::default(),
				},
				proficiency::Level::Full,
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn skill_any_withlevel() -> anyhow::Result<()> {
			let doc =
				"mutator \"add_proficiency\" (Skill)\"Any\" id=\"MutatorSelect\" level=\"Half\"";
			let expected = AddProficiency::Skill(
				Selector::Any {
					id: Some("MutatorSelect").into(),
					cannot_match: Default::default(),
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
					id: Some("MutatorSelect").into(),
					options: vec![Skill::Insight, Skill::AnimalHandling],
					cannot_match: Default::default(),
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
					id: Default::default(),
					options: vec![Skill::Insight, Skill::AnimalHandling],
					cannot_match: Default::default(),
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
			let expected = AddProficiency::Language(Selector::Any {
				id: Default::default(),
				cannot_match: Default::default(),
			});
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
				id: Default::default(),
				options: vec!["Dwarven".into(), "Giant".into()],
				cannot_match: Default::default(),
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn armor_kind() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Medium\"";
			let expected = AddProficiency::Armor(ArmorExtended::Kind(armor::Kind::Medium), None);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn armor_kind_ctx() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Medium\" \"nonmetal\"";
			let expected = AddProficiency::Armor(
				ArmorExtended::Kind(armor::Kind::Medium),
				Some("nonmetal".into()),
			);
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn armor_shield() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Shield\"";
			let expected = AddProficiency::Armor(ArmorExtended::Shield, None);
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
		fn tool_specific() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"Specific\" \"Dragonchess Set\"";
			let expected = AddProficiency::Tool(Selector::Specific("Dragonchess Set".into()));
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn tool_any() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"Any\"";
			let expected = AddProficiency::Tool(Selector::Any {
				id: Default::default(),
				cannot_match: Default::default(),
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn tool_anyof() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"AnyOf\" {
				option \"Dice set\"
				option \"Playing card set\"
				option \"Flute\"
			}";
			let expected = AddProficiency::Tool(Selector::AnyOf {
				id: Default::default(),
				options: vec!["Dice set".into(), "Playing card set".into(), "Flute".into()],
				cannot_match: Default::default(),
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::{
			path_map::PathMap,
			system::dnd5e::data::{
				character::{Character, Persistent},
				Feature,
			},
		};
		use std::path::PathBuf;

		fn character(mutator: AddProficiency, selections: Option<PathMap<String>>) -> Character {
			Character::from(Persistent {
				feats: vec![Feature {
					name: "AddProficiency".into(),
					mutators: vec![mutator.into()],
					..Default::default()
				}
				.into()],
				selected_values: selections.unwrap_or_default(),
				..Default::default()
			})
		}

		#[test]
		fn saving_throw() {
			let character = character(AddProficiency::SavingThrow(Ability::Dexterity), None);
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
			let character = character(
				AddProficiency::Skill(
					Selector::Specific(Skill::Arcana),
					proficiency::Level::Double,
				),
				None,
			);
			assert_eq!(
				*character.skills().proficiency(Skill::Arcana),
				(
					proficiency::Level::Double,
					vec![("AddProficiency".into(), proficiency::Level::Double)]
				)
					.into(),
			);
		}

		#[test]
		fn language() {
			let character = character(
				AddProficiency::Language(Selector::Specific("Common".into())),
				None,
			);
			assert_eq!(
				*character.other_proficiencies().languages,
				[("Common".into(), ["AddProficiency".into()].into())].into()
			);
		}

		#[test]
		fn language_any() {
			let character = character(
				AddProficiency::Language(Selector::Any {
					id: Some("langTest").into(),
					cannot_match: Default::default(),
				}),
				Some([("AddProficiency/langTest", "Gibberish".into())].into()),
			);
			assert_eq!(
				character.missing_selections_in(PathBuf::new()),
				Vec::<&std::path::Path>::new()
			);
			assert_eq!(
				*character.other_proficiencies().languages,
				[("Gibberish".into(), ["AddProficiency".into()].into())].into()
			);
		}

		#[test]
		fn armor_kind() {
			let character = character(
				AddProficiency::Armor(ArmorExtended::Kind(armor::Kind::Heavy), None),
				None,
			);
			assert_eq!(
				*character.other_proficiencies().armor,
				[(
					(ArmorExtended::Kind(armor::Kind::Heavy), None),
					["AddProficiency".into()].into()
				)]
				.into()
			);
		}

		#[test]
		fn armor_kind_ctx() {
			let character = character(
				AddProficiency::Armor(
					ArmorExtended::Kind(armor::Kind::Heavy),
					Some("nonmetal".into()),
				),
				None,
			);
			assert_eq!(
				*character.other_proficiencies().armor,
				[(
					(
						ArmorExtended::Kind(armor::Kind::Heavy),
						Some("nonmetal".into())
					),
					["AddProficiency".into()].into()
				)]
				.into()
			);
		}

		#[test]
		fn armor_shield() {
			let character = character(AddProficiency::Armor(ArmorExtended::Shield, None), None);
			assert_eq!(
				*character.other_proficiencies().armor,
				[(
					(ArmorExtended::Shield, None),
					["AddProficiency".into()].into()
				)]
				.into()
			);
		}

		#[test]
		fn weapon_kind() {
			let character = character(
				AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial)),
				None,
			);
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
			let character = character(
				AddProficiency::Weapon(WeaponProficiency::Classification("Quarterstaff".into())),
				None,
			);
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
		fn tool_specific() {
			let character = character(
				AddProficiency::Tool(Selector::Specific("Thieves' Tools".into())),
				None,
			);
			assert_eq!(
				*character.other_proficiencies().tools,
				[("Thieves' Tools".into(), ["AddProficiency".into()].into())].into()
			);
		}
	}
}
