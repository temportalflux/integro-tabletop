use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, ValueExt},
	system::dnd5e::data::{
		character::Character, description, item::weapon, proficiency, Ability, ArmorExtended,
		Skill, WeaponProficiency,
	},
	utility::{selector, Mutator, NotInList},
};
use enumset::EnumSet;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum AddProficiency {
	Ability(selector::Value<Character, Ability>, proficiency::Level),
	SavingThrow(Ability),
	Skill(selector::Value<Character, Skill>, proficiency::Level),
	Language(selector::Value<Character, String>),
	Armor(ArmorExtended, Option<String>),
	Weapon(WeaponProficiency),
	Tool(selector::Value<Character, String>),
}

crate::impl_trait_eq!(AddProficiency);
crate::impl_kdl_node!(AddProficiency, "add_proficiency");

impl Mutator for AddProficiency {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		let content = match self {
			Self::Ability(selector::Value::Specific(ability), level) => format!(
				"You are {} with all skill checks which use {}.",
				level.as_display_name().to_lowercase(),
				ability.long_name()
			),
			Self::Ability(selector::Value::Options { options, .. }, level)
				if options.is_empty() =>
			{
				format!(
					"You are {} with all skill checks which use one ability of your choice.",
					level.as_display_name().to_lowercase()
				)
			}
			Self::Ability(selector::Value::Options { options, .. }, level) => format!(
				"You are {} with all skill checks which use one ability of: {}.",
				level.as_display_name().to_lowercase(),
				options
					.iter()
					.map(Ability::long_name)
					.collect::<Vec<_>>()
					.join(", ")
			),
			Self::SavingThrow(ability) => format!(
				"You are proficient with {} saving throws.",
				ability.long_name()
			),
			Self::Skill(selector::Value::Specific(skill), level) => format!(
				"You are {} with {} checks.",
				level.as_display_name().to_lowercase(),
				skill.display_name()
			),
			Self::Skill(selector::Value::Options { options, .. }, level) if options.is_empty() => {
				format!(
					"You are {} with one skill of your choice.",
					level.as_display_name().to_lowercase()
				)
			}
			Self::Skill(selector::Value::Options { options, .. }, level) => format!(
				"You are {} with one skill of: {}.",
				level.as_display_name().to_lowercase(),
				options
					.iter()
					.map(Skill::display_name)
					.collect::<Vec<_>>()
					.join(", ")
			),
			Self::Language(selector::Value::Specific(lang)) => {
				format!("You can speak, read, and write {lang}.")
			}
			Self::Language(selector::Value::Options { options, .. }) if options.is_empty() => {
				format!("You can speak, read, and write one language of your choice.")
			}
			Self::Language(selector::Value::Options { options, .. }) => format!(
				"You can speak, read, and write one language of: {}.",
				options.iter().cloned().collect::<Vec<_>>().join(", ")
			),
			Self::Armor(kind, context) => {
				let ctx = context
					.as_ref()
					.map(|s| format!(" ({s})"))
					.unwrap_or_default();
				match kind {
					ArmorExtended::Kind(kind) => format!(
						"You are proficient with {} armor{ctx}.",
						kind.to_string().to_lowercase()
					),
					ArmorExtended::Shield => format!("You are proficient with shields{ctx}."),
				}
			}
			Self::Weapon(WeaponProficiency::Kind(kind)) => format!(
				"You are proficient with {} weapons.",
				kind.to_string().to_lowercase()
			),
			Self::Weapon(WeaponProficiency::Classification(kind)) => {
				format!("You are proficient with {kind} weapon-types.")
			}
			Self::Tool(selector::Value::Specific(tool)) => {
				format!("You are proficient with {tool}.")
			}
			Self::Tool(selector::Value::Options { options, .. }) if options.is_empty() => {
				format!("You are proficient with one tool of your choice.")
			}
			Self::Tool(selector::Value::Options { options, .. }) => format!(
				"You are proficient with one tool of: {}.",
				options.iter().cloned().collect::<Vec<_>>().join(", ")
			),
		};
		let selectors = match self {
			Self::Skill(selector, _) => {
				selector::DataList::default().with_enum("Skill", selector, state)
			}
			Self::Language(selector) => {
				selector::DataList::default().with_value("Language", selector, state)
			}
			Self::Tool(selector) => {
				selector::DataList::default().with_value("Tool", selector, state)
			}
			_ => Default::default(),
		};
		description::Section {
			content: content.into(),
			children: vec![selectors.into()],
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		match self {
			Self::Ability(selector, _) => selector.set_data_path(parent),
			Self::Skill(selector, _) => selector.set_data_path(parent),
			Self::Language(selector) => selector.set_data_path(parent),
			Self::Tool(selector) => selector.set_data_path(parent),
			_ => {}
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		match &self {
			Self::Ability(ability, level) => {
				if let Some(ability) = stats.resolve_selector(ability) {
					// TODO: Grant proficiency for an ability (in addition to the skills which use that ability)
					let derived_skills = stats.skills_mut();
					log::debug!("{level:?} in {ability:?}");
					for skill in EnumSet::<Skill>::all() {
						if skill.ability() == ability {
							log::debug!("{level:?} in {skill:?}");
							derived_skills.add_proficiency(skill, *level, parent.to_owned());
						}
					}
				}
			}
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
}

impl FromKDL for AddProficiency {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.peak_type_req()? {
			"Ability" => {
				let ability = selector::Value::from_kdl(node)?;
				let level = match node.get_str_opt("level")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::Full,
				};
				Ok(Self::Ability(ability, level))
			}
			"SavingThrow" => Ok(Self::SavingThrow(node.next_str_req_t::<Ability>()?)),
			"Skill" => {
				let skill = selector::Value::from_kdl(node)?;
				let level = match node.get_str_opt("level")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::Full,
				};
				Ok(Self::Skill(skill, level))
			}
			"Language" => Ok(Self::Language(selector::Value::from_kdl(node)?)),
			"Armor" => {
				let kind = node.next_str_req_t::<ArmorExtended>()?;
				let context = node.next_str_opt()?.map(str::to_owned);
				Ok(Self::Armor(kind, context))
			}
			"Weapon" => {
				let entry = node.next_req()?;
				Ok(Self::Weapon(match entry.as_str_req()? {
					kind if kind == "Simple" || kind == "Martial" => {
						WeaponProficiency::Kind(weapon::Kind::from_str(kind)?)
					}
					classification => WeaponProficiency::Classification(classification.to_owned()),
				}))
			}
			"Tool" => Ok(Self::Tool(selector::Value::from_kdl(node)?)),
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

impl AsKdl for AddProficiency {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Ability(ability, level) => {
				node.append_typed("Ability", ability.as_kdl());
				if *level != proficiency::Level::Full {
					node.push_entry(("level", level.to_string()));
				}
				node
			}
			Self::SavingThrow(ability) => node.with_entry_typed(ability.long_name(), "SavingThrow"),
			Self::Skill(skill, level) => {
				node.append_typed("Skill", skill.as_kdl());
				if *level != proficiency::Level::Full {
					node.push_entry(("level", level.to_string()));
				}
				node
			}
			Self::Language(lang_name) => {
				node.append_typed("Language", lang_name.as_kdl());
				node
			}
			Self::Armor(armor_ext, context) => {
				node.push_entry_typed(armor_ext.to_string(), "Armor");
				if let Some(context) = context {
					node.push_entry(context.clone());
				}
				node
			}
			Self::Weapon(weapon_prof) => node.with_entry_typed(weapon_prof.to_string(), "Weapon"),
			Self::Tool(tool_name) => {
				node.append_typed("Tool", tool_name.as_kdl());
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::item::{armor, weapon};

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils, utility::Value,
		};

		test_utils!(AddProficiency);

		#[test]
		fn ability_specific_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Ability)\"Specific\" \"Intelligence\"";
			let data = AddProficiency::Ability(
				selector::Value::Specific(Ability::Intelligence),
				proficiency::Level::Full,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn ability_specific_withlevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Ability)\"Specific\" \"Wisdom\" level=\"Double\"";
			let data = AddProficiency::Ability(
				selector::Value::Specific(Ability::Wisdom),
				proficiency::Level::Double,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn ability_any_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Ability)\"Any\" id=\"MutatorSelect\"";
			let data = AddProficiency::Ability(
				selector::Value::Options {
					id: Some("MutatorSelect").into(),
					options: Default::default(),
					amount: Value::Fixed(1),
					is_applicable: None,
				},
				proficiency::Level::Full,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn saving_throw() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (SavingThrow)\"Constitution\"";
			let data = AddProficiency::SavingThrow(Ability::Constitution);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_specific_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Specific\" \"Insight\"";
			let data = AddProficiency::Skill(
				selector::Value::Specific(Skill::Insight),
				proficiency::Level::Full,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_specific_withlevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Skill)\"Specific\" \"Religion\" level=\"Double\"";
			let data = AddProficiency::Skill(
				selector::Value::Specific(Skill::Religion),
				proficiency::Level::Double,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\" id=\"MutatorSelect\"";
			let data = AddProficiency::Skill(
				selector::Value::Options {
					id: Some("MutatorSelect").into(),
					options: Default::default(),
					amount: Value::Fixed(1),
					is_applicable: None,
				},
				proficiency::Level::Full,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel_noid() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\"";
			let data = AddProficiency::Skill(
				selector::Value::Options {
					id: Default::default(),
					options: Default::default(),
					amount: Value::Fixed(1),
					is_applicable: None,
				},
				proficiency::Level::Full,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_any_withlevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Skill)\"Any\" id=\"MutatorSelect\" level=\"HalfDown\"";
			let data = AddProficiency::Skill(
				selector::Value::Options {
					id: Some("MutatorSelect").into(),
					options: Default::default(),
					amount: Value::Fixed(1),
					is_applicable: None,
				},
				proficiency::Level::HalfDown,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_anyof_nolevel() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Skill)\"AnyOf\" id=\"MutatorSelect\" {
				|    option \"Insight\"
				|    option \"AnimalHandling\"
				|}
			";
			let data = AddProficiency::Skill(
				selector::Value::Options {
					id: Some("MutatorSelect").into(),
					options: [Skill::Insight, Skill::AnimalHandling].into(),
					amount: Value::Fixed(1),
					is_applicable: None,
				},
				proficiency::Level::Full,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_anyof_withlevel_noid() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Skill)\"AnyOf\" level=\"Double\" {
				|    option \"Insight\"
				|    option \"AnimalHandling\"
				|}
			";
			let data = AddProficiency::Skill(
				selector::Value::Options {
					id: Default::default(),
					options: [Skill::Insight, Skill::AnimalHandling].into(),
					amount: Value::Fixed(1),
					is_applicable: None,
				},
				proficiency::Level::Double,
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn language_specific() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Language)\"Specific\" \"Gibberish\"";
			let data = AddProficiency::Language(selector::Value::Specific("Gibberish".into()));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn language_any() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Language)\"Any\"";
			let data = AddProficiency::Language(selector::Value::Options {
				id: Default::default(),
				options: Default::default(),
				amount: Value::Fixed(1),
				is_applicable: None,
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn language_anyof() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Language)\"AnyOf\" {
				|    option \"Dwarven\"
				|    option \"Giant\"
				|}
			";
			let data = AddProficiency::Language(selector::Value::Options {
				id: Default::default(),
				options: ["Dwarven".into(), "Giant".into()].into(),
				amount: Value::Fixed(1),
				is_applicable: None,
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn armor_kind() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Medium\"";
			let data = AddProficiency::Armor(ArmorExtended::Kind(armor::Kind::Medium), None);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn armor_kind_ctx() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Medium\" \"nonmetal\"";
			let data = AddProficiency::Armor(
				ArmorExtended::Kind(armor::Kind::Medium),
				Some("nonmetal".into()),
			);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn armor_shield() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Armor)\"Shield\"";
			let data = AddProficiency::Armor(ArmorExtended::Shield, None);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn weapon_simple() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Weapon)\"Simple\"";
			let data = AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Simple));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn weapon_martial() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Weapon)\"Martial\"";
			let data = AddProficiency::Weapon(WeaponProficiency::Kind(weapon::Kind::Martial));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn weapon_class() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Weapon)\"Club\"";
			let data = AddProficiency::Weapon(WeaponProficiency::Classification("Club".into()));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn tool_specific() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"Specific\" \"Dragonchess Set\"";
			let data = AddProficiency::Tool(selector::Value::Specific("Dragonchess Set".into()));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn tool_any() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"Any\"";
			let data = AddProficiency::Tool(selector::Value::Options {
				id: Default::default(),
				options: Default::default(),
				amount: Value::Fixed(1),
				is_applicable: None,
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn tool_anyof() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Tool)\"AnyOf\" {
				|    option \"Dice set\"
				|    option \"Playing card set\"
				|    option \"Flute\"
				|}
			";
			let data = AddProficiency::Tool(selector::Value::Options {
				id: Default::default(),
				options: ["Dice set".into(), "Playing card set".into(), "Flute".into()].into(),
				amount: Value::Fixed(1),
				is_applicable: None,
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::{
			path_map::PathMap,
			system::dnd5e::data::{
				character::{AttributedValue, Character, Persistent},
				Bundle,
			},
			utility::Value,
		};
		use std::path::PathBuf;

		fn character(mutator: AddProficiency, selections: Option<PathMap<String>>) -> Character {
			Character::from(Persistent {
				bundles: vec![Bundle {
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
		fn ability() {
			let character = character(
				AddProficiency::Ability(
					selector::Value::Specific(Ability::Intelligence),
					proficiency::Level::Full,
				),
				None,
			);
			let exepected_prof: AttributedValue<proficiency::Level> = (
				proficiency::Level::Full,
				vec![("AddProficiency".into(), proficiency::Level::Full)],
			)
				.into();
			for skill in EnumSet::<Skill>::all() {
				if skill.ability() != Ability::Intelligence {
					continue;
				}
				let prof = character.skills().proficiency(skill);
				assert_eq!(*prof, exepected_prof);
			}
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
					selector::Value::Specific(Skill::Arcana),
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
				AddProficiency::Language(selector::Value::Specific("Common".into())),
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
				AddProficiency::Language(selector::Value::Options {
					id: Some("langTest").into(),
					options: Default::default(),
					amount: Value::Fixed(1),
					is_applicable: None,
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
				AddProficiency::Tool(selector::Value::Specific("Thieves' Tools".into())),
				None,
			);
			assert_eq!(
				*character.other_proficiencies().tools,
				[("Thieves' Tools".into(), ["AddProficiency".into()].into())].into()
			);
		}
	}
}
