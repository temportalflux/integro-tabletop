use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::data::{
		character::Character, description, item::weapon, proficiency, Ability, ArmorExtended, Skill, WeaponProficiency,
	},
	system::Mutator,
	utility::{selector, NotInList},
};
use enumset::EnumSet;
use kdlize::{ext::ValueExt, AsKdl, FromKdl, NodeBuilder};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum AddProficiency {
	Ability(selector::Value<Character, Ability>, proficiency::Level),
	SavingThrow(Ability),
	Skill {
		// The skill to grant proficiency to
		skill: selector::Value<Character, Skill>,
		// The minimum level the skill has to be in order to grant proficiency
		minimum_level: proficiency::Level,
		// The proficiency to grant to the skill
		level: proficiency::Level,
	},
	Language(selector::Value<Character, String>),
	Armor(ArmorExtended, Option<String>),
	Weapon(WeaponProficiency),
	Tool {
		tool: selector::Value<Character, String>,
		level: proficiency::Level,
	},
}

crate::impl_trait_eq!(AddProficiency);
kdlize::impl_kdl_node!(AddProficiency, "add_proficiency");

impl Mutator for AddProficiency {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		let content = match self {
			Self::Ability(selector::Value::Specific(ability), level) => format!(
				"You are {} with all skill checks which use {}.",
				level.as_display_name().to_lowercase(),
				ability.long_name()
			),
			Self::Ability(selector::Value::Options(selector::ValueOptions { options, .. }), level)
				if options.is_empty() =>
			{
				format!(
					"You are {} with all skill checks which use one ability of your choice.",
					level.as_display_name().to_lowercase()
				)
			}
			Self::Ability(selector::Value::Options(selector::ValueOptions { options, .. }), level) => format!(
				"You are {} with all skill checks which use one ability of: {}.",
				level.as_display_name().to_lowercase(),
				options.iter().map(Ability::long_name).collect::<Vec<_>>().join(", ")
			),
			Self::SavingThrow(ability) => format!("You are proficient with {} saving throws.", ability.long_name()),
			Self::Skill {
				skill: selector::Value::Specific(skill),
				level,
				minimum_level: _,
			} => format!(
				"You are {} with {} checks.",
				level.as_display_name().to_lowercase(),
				skill.display_name()
			),
			Self::Skill {
				skill: selector::Value::Options(selector::ValueOptions { options, .. }),
				level,
				minimum_level: _,
			} if options.is_empty() => {
				format!(
					"You are {} with one skill of your choice.",
					level.as_display_name().to_lowercase()
				)
			}
			Self::Skill {
				skill: selector::Value::Options(selector::ValueOptions { options, .. }),
				level,
				minimum_level: _,
			} => format!(
				"You are {} with one skill of: {}.",
				level.as_display_name().to_lowercase(),
				options.iter().map(Skill::display_name).collect::<Vec<_>>().join(", ")
			),
			Self::Language(selector::Value::Specific(lang)) => {
				format!("You can speak, read, and write {lang}.")
			}
			Self::Language(selector::Value::Options(selector::ValueOptions { options, .. })) if options.is_empty() => {
				format!("You can speak, read, and write one language of your choice.")
			}
			Self::Language(selector::Value::Options(selector::ValueOptions { options, .. })) => format!(
				"You can speak, read, and write one language of: {}.",
				options.iter().cloned().collect::<Vec<_>>().join(", ")
			),
			Self::Armor(kind, context) => {
				let ctx = context.as_ref().map(|s| format!(" ({s})")).unwrap_or_default();
				match kind {
					ArmorExtended::Kind(kind) => format!(
						"You are proficient with {} armor{ctx}.",
						kind.to_string().to_lowercase()
					),
					ArmorExtended::Shield => format!("You are proficient with shields{ctx}."),
				}
			}
			Self::Weapon(WeaponProficiency::Kind(kind)) => {
				format!("You are proficient with {} weapons.", kind.to_string().to_lowercase())
			}
			Self::Weapon(WeaponProficiency::Classification(kind)) => {
				format!("You are proficient with {kind} weapon-types.")
			}
			Self::Tool {
				tool: selector::Value::Specific(tool),
				level,
			} => {
				format!("You are {} with {tool}.", level.as_display_name().to_lowercase())
			}
			Self::Tool {
				tool: selector::Value::Options(selector::ValueOptions { options, .. }),
				level,
			} if options.is_empty() => {
				format!(
					"You are {} with one tool of your choice.",
					level.as_display_name().to_lowercase()
				)
			}
			Self::Tool {
				tool: selector::Value::Options(selector::ValueOptions { options, .. }),
				level,
			} => format!(
				"You are {} with one tool of: {}.",
				level.as_display_name().to_lowercase(),
				options.iter().cloned().collect::<Vec<_>>().join(", ")
			),
		};
		let selectors = match self {
			Self::Skill { skill, .. } => selector::DataList::default().with_enum("Skill", skill, state),
			Self::Language(selector) => selector::DataList::default().with_value("Language", selector, state),
			Self::Tool { tool, .. } => selector::DataList::default().with_value("Tool", tool, state),
			_ => Default::default(),
		};
		description::Section {
			content: content.into(),
			children: vec![selectors.into()],
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &ReferencePath) {
		match self {
			Self::Ability(selector, _) => selector.set_data_path(parent),
			Self::Skill { skill, .. } => skill.set_data_path(parent),
			Self::Language(selector) => selector.set_data_path(parent),
			Self::Tool { tool, .. } => tool.set_data_path(parent),
			_ => {}
		}
	}

	fn on_insert(&self, stats: &mut Character, parent: &ReferencePath) {
		match &self {
			Self::Ability(ability, level) => {
				if let Some(ability) = stats.resolve_selector(ability) {
					// TODO: Grant proficiency for an ability (in addition to the skills which use that ability)
					let derived_skills = stats.skills_mut();
					for skill in EnumSet::<Skill>::all() {
						if skill.ability() == ability {
							derived_skills.add_proficiency(skill, *level, parent);
						}
					}
				}
			}
			Self::SavingThrow(ability) => {
				stats.saving_throws_mut().add_proficiency(*ability, parent);
			}
			Self::Skill {
				skill,
				level,
				minimum_level: _,
			} => {
				let Some(skill) = stats.resolve_selector(skill) else {
					return;
				};
				stats.skills_mut().add_proficiency(skill, *level, parent);
			}
			Self::Language(value) => {
				if let Some(value) = stats.resolve_selector(value) {
					stats.other_proficiencies_mut().languages.insert(value, parent);
				}
			}
			Self::Armor(value, context) => {
				stats
					.other_proficiencies_mut()
					.armor
					.insert((value.clone(), context.clone()), parent);
			}
			Self::Weapon(value) => {
				stats.other_proficiencies_mut().weapons.insert(value.clone(), parent);
			}
			Self::Tool { tool, level: _ } => {
				// TODO: Actually grant the tool's proficiency level (there is no place to put that data yet)
				if let Some(value) = stats.resolve_selector(tool) {
					stats.other_proficiencies_mut().tools.insert(value, parent);
				}
			}
		}
	}
}

impl FromKdl<NodeContext> for AddProficiency {
	type Error = anyhow::Error;
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
				let mut skill = selector::Value::from_kdl(node)?;
				let level = match node.get_str_opt("level")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::Full,
				};

				let minimum_level = match node.get_str_opt("min")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::None,
				};
				skill.set_is_applicable(move |skill, character| {
					let active_level = *character.skills().proficiency(*skill).value();
					if active_level < minimum_level {
						return false;
					}
					active_level < level
				});

				Ok(Self::Skill {
					skill,
					level,
					minimum_level,
				})
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
			"Tool" => {
				let tool = selector::Value::from_kdl(node)?;
				let level = match node.get_str_opt("level")? {
					Some(str) => proficiency::Level::from_str(str)?,
					None => proficiency::Level::Full,
				};
				Ok(Self::Tool { tool, level })
			}
			name => Err(NotInList(
				name.into(),
				vec!["SavingThrow", "Skill", "Language", "Armor", "Weapon", "Tool"],
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
				node += ("Ability", ability.as_kdl());
				if *level != proficiency::Level::Full {
					node.entry(("level", level.to_string()));
				}
				node
			}
			Self::SavingThrow(ability) => node.with_entry_typed(ability.long_name(), "SavingThrow"),
			Self::Skill {
				skill,
				level,
				minimum_level,
			} => {
				node += ("Skill", skill.as_kdl());
				if *level != proficiency::Level::Full {
					node.entry(("level", level.to_string()));
				}
				if *minimum_level != proficiency::Level::None {
					node.entry(("min", minimum_level.to_string()));
				}
				node
			}
			Self::Language(lang_name) => {
				node += ("Language", lang_name.as_kdl());
				node
			}
			Self::Armor(armor_ext, context) => {
				node.entry_typed("Armor", armor_ext.to_string());
				if let Some(context) = context {
					node.entry(context.clone());
				}
				node
			}
			Self::Weapon(weapon_prof) => node.with_entry_typed(weapon_prof.to_string(), "Weapon"),
			Self::Tool { tool, level } => {
				node += ("Tool", tool.as_kdl());
				if *level != proficiency::Level::Full {
					node.entry(("level", level.to_string()));
				}
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
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

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
			let data = AddProficiency::Ability(selector::Value::Specific(Ability::Wisdom), proficiency::Level::Double);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn ability_any_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Ability)\"Any\" id=\"MutatorSelect\"";
			let data = AddProficiency::Ability(
				selector::Value::Options(selector::ValueOptions {
					id: Some("MutatorSelect").into(),
					..Default::default()
				}),
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
			let data = AddProficiency::Skill {
				skill: selector::Value::Specific(Skill::Insight),
				level: proficiency::Level::Full,
				minimum_level: proficiency::Level::None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_specific_withlevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Skill)\"Specific\" \"Religion\" level=\"Double\"";
			let data = AddProficiency::Skill {
				skill: selector::Value::Specific(Skill::Religion),
				level: proficiency::Level::Double,
				minimum_level: proficiency::Level::None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\" id=\"MutatorSelect\"";
			let data = AddProficiency::Skill {
				skill: selector::Value::Options(selector::ValueOptions {
					id: Some("MutatorSelect").into(),
					..Default::default()
				}),
				level: proficiency::Level::Full,
				minimum_level: proficiency::Level::None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_any_nolevel_noid() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Skill)\"Any\"";
			let data = AddProficiency::Skill {
				skill: selector::Value::Options(selector::ValueOptions::default()),
				level: proficiency::Level::Full,
				minimum_level: proficiency::Level::None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_any_withlevel() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" \
				(Skill)\"Any\" id=\"MutatorSelect\" level=\"HalfDown\"";
			let data = AddProficiency::Skill {
				skill: selector::Value::Options(selector::ValueOptions {
					id: Some("MutatorSelect").into(),
					..Default::default()
				}),
				level: proficiency::Level::HalfDown,
				minimum_level: proficiency::Level::None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_anyof_nolevel() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Skill)\"Any\" id=\"MutatorSelect\" {
				|    option \"AnimalHandling\"
				|    option \"Insight\"
				|}
			";
			let data = AddProficiency::Skill {
				skill: selector::Value::Options(selector::ValueOptions {
					id: Some("MutatorSelect").into(),
					options: [Skill::Insight, Skill::AnimalHandling].into(),
					..Default::default()
				}),
				level: proficiency::Level::Full,
				minimum_level: proficiency::Level::None,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill_anyof_withlevel_noid() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Skill)\"Any\" level=\"Double\" {
				|    option \"AnimalHandling\"
				|    option \"Insight\"
				|}
			";
			let data = AddProficiency::Skill {
				skill: selector::Value::Options(selector::ValueOptions {
					options: [Skill::Insight, Skill::AnimalHandling].into(),
					..Default::default()
				}),
				level: proficiency::Level::Double,
				minimum_level: proficiency::Level::None,
			};
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
			let data = AddProficiency::Language(selector::Value::Options(selector::ValueOptions::default()));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn language_anyof() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Language)\"Any\" {
				|    option \"Dwarven\"
				|    option \"Giant\"
				|}
			";
			let data = AddProficiency::Language(selector::Value::Options(selector::ValueOptions {
				options: ["Dwarven".into(), "Giant".into()].into(),
				..Default::default()
			}));
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
			let data = AddProficiency::Armor(ArmorExtended::Kind(armor::Kind::Medium), Some("nonmetal".into()));
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
			let data = AddProficiency::Tool {
				tool: selector::Value::Specific("Dragonchess Set".into()),
				level: proficiency::Level::Full,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn tool_any() -> anyhow::Result<()> {
			let doc = "mutator \"add_proficiency\" (Tool)\"Any\"";
			let data = AddProficiency::Tool {
				tool: selector::Value::Options(selector::ValueOptions::default()),
				level: proficiency::Level::Full,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn tool_anyof() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_proficiency\" (Tool)\"Any\" {
				|    option \"Dice set\"
				|    option \"Flute\"
				|    option \"Playing card set\"
				|}
			";
			let data = AddProficiency::Tool {
				tool: selector::Value::Options(selector::ValueOptions {
					options: ["Dice set".into(), "Playing card set".into(), "Flute".into()].into(),
					..Default::default()
				}),
				level: proficiency::Level::Full,
			};
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
				*character.saving_throws().get_prof(Ability::Dexterity).value(),
				proficiency::Level::Full,
			);
		}

		#[test]
		fn skill() {
			let character = character(
				AddProficiency::Skill {
					skill: selector::Value::Specific(Skill::Arcana),
					level: proficiency::Level::Double,
					minimum_level: proficiency::Level::None,
				},
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
				AddProficiency::Language(selector::Value::Options(selector::ValueOptions {
					id: Some("langTest").into(),
					..Default::default()
				})),
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
				AddProficiency::Armor(ArmorExtended::Kind(armor::Kind::Heavy), Some("nonmetal".into())),
				None,
			);
			assert_eq!(
				*character.other_proficiencies().armor,
				[(
					(ArmorExtended::Kind(armor::Kind::Heavy), Some("nonmetal".into())),
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
				[((ArmorExtended::Shield, None), ["AddProficiency".into()].into())].into()
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
				AddProficiency::Tool {
					tool: selector::Value::Specific("Thieves' Tools".into()),
					level: proficiency::Level::Full,
				},
				None,
			);
			assert_eq!(
				*character.other_proficiencies().tools,
				[("Thieves' Tools".into(), ["AddProficiency".into()].into())].into()
			);
		}
	}
}
