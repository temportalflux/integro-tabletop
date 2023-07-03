use crate::{
	kdl_ext::{AsKdl, EntryExt, FromKDL, NodeBuilder, NodeExt, ValueExt},
	system::dnd5e::data::{character::Character, description, roll, Ability, Skill},
	utility::{Mutator, NotInList, Selector, SelectorMetaVec},
};
use std::{path::Path, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub struct AddModifier {
	pub modifier: roll::Modifier,
	pub context: Option<String>,
	pub kind: ModifierKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierKind {
	Ability(Selector<Ability>),
	SavingThrow(Option<Selector<Ability>>),
	Skill(Selector<Skill>),
	Initiative,
}

crate::impl_trait_eq!(AddModifier);
crate::impl_kdl_node!(AddModifier, "add_modifier");

impl Mutator for AddModifier {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		let mut desc = format!("You have {} on ", self.modifier.display_name());
		let kind_desc = match &self.kind {
			ModifierKind::Ability(Selector::Specific(ability)) => {
				format!("{} checks", ability.long_name())
			}
			ModifierKind::Ability(Selector::AnyOf { options, .. }) => format!(
				"any single ability check (of: {})",
				options
					.iter()
					.map(Ability::long_name)
					.collect::<Vec<_>>()
					.join(", ")
			),
			ModifierKind::Ability(Selector::Any { .. }) => {
				format!("any single ability check")
			}
			ModifierKind::SavingThrow(Some(Selector::Specific(ability))) => {
				format!("{} saving throws", ability.long_name(),)
			}
			ModifierKind::SavingThrow(Some(Selector::AnyOf { options, .. })) => format!(
				"any single ability saving throw (of: {})",
				options
					.iter()
					.map(Ability::long_name)
					.collect::<Vec<_>>()
					.join(", ")
			),
			ModifierKind::SavingThrow(Some(Selector::Any { .. })) => {
				format!("any single ability saving throw")
			}
			ModifierKind::SavingThrow(None) => format!("saving throws"),
			ModifierKind::Skill(Selector::Specific(skill)) => format!(
				"{} ({}) checks",
				skill.ability().long_name(),
				skill.display_name()
			),
			ModifierKind::Skill(Selector::AnyOf { options, .. }) => format!(
				"any single ability skill check (of: {})",
				options
					.iter()
					.map(Skill::display_name)
					.collect::<Vec<_>>()
					.join(", ")
			),
			ModifierKind::Skill(Selector::Any { .. }) => {
				format!("any single ability skill check")
			}
			ModifierKind::Initiative => {
				format!("initiative checks")
			}
		};
		desc.push_str(&kind_desc);
		if let Some(ctx) = &self.context {
			desc.push_str(match &self.kind {
				ModifierKind::Ability(_) => "",
				ModifierKind::SavingThrow(_) => " against",
				ModifierKind::Skill(_) => "",
				ModifierKind::Initiative => "",
			});
			desc.push(' ');
			desc.push_str(ctx.as_str());
		}
		desc.push('.');
		let selectors = match &self.kind {
			ModifierKind::Ability(selector) => {
				SelectorMetaVec::default().with_enum("Ability", selector)
			}
			ModifierKind::SavingThrow(Some(selector)) => {
				SelectorMetaVec::default().with_enum("Ability", selector)
			}
			ModifierKind::SavingThrow(None) => Default::default(),
			ModifierKind::Skill(selector) => {
				SelectorMetaVec::default().with_enum("Skill", selector)
			}
			ModifierKind::Initiative => Default::default(),
		};
		description::Section {
			content: desc.into(),
			children: vec![selectors.into()],
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		match &self.kind {
			ModifierKind::Ability(selector) => selector.set_data_path(parent),
			ModifierKind::SavingThrow(Some(selector)) => selector.set_data_path(parent),
			ModifierKind::SavingThrow(None) => {}
			ModifierKind::Skill(selector) => selector.set_data_path(parent),
			ModifierKind::Initiative => {}
		}
	}

	fn apply(&self, stats: &mut Character, parent: &Path) {
		match &self.kind {
			ModifierKind::Ability(ability) => {
				let Some(ability) = stats.resolve_selector(ability) else { return; };
				stats.skills_mut().add_ability_modifier(
					ability,
					self.modifier,
					self.context.clone(),
					parent.to_owned(),
				);
			}
			ModifierKind::SavingThrow(ability) => {
				let ability = match ability {
					None => None,
					Some(ability) => stats.resolve_selector(ability),
				};
				stats.saving_throws_mut().add_modifier(
					ability,
					self.modifier,
					self.context.clone(),
					parent.to_owned(),
				);
			}
			ModifierKind::Skill(skill) => {
				let Some(skill) = stats.resolve_selector(skill) else { return; };
				stats.skills_mut().add_skill_modifier(
					skill,
					self.modifier,
					self.context.clone(),
					parent.to_owned(),
				);
			}
			ModifierKind::Initiative => {
				// TODO: apply advantage or disadvantage to initiative
			}
		}
	}
}

impl FromKDL for AddModifier {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let modifier = roll::Modifier::from_str(node.next_str_req()?)?;
		let context = node.get_str_opt("context")?.map(str::to_owned);
		let entry = node.next_req()?;
		let kind = match entry.type_req()? {
			"Ability" => {
				let ability = Selector::from_kdl(node, entry, |kdl| {
					Ok(Ability::from_str(kdl.as_str_req()?)?)
				})?;
				ModifierKind::Ability(ability)
			}
			"SavingThrow" => {
				let ability = match entry.as_str_req()? {
					"All" => None,
					_ => Some(Selector::from_kdl(node, entry, |kdl| {
						Ok(Ability::from_str(kdl.as_str_req()?)?)
					})?),
				};
				ModifierKind::SavingThrow(ability)
			}
			"Skill" => {
				let skill =
					Selector::from_kdl(node, entry, |kdl| Ok(Skill::from_str(kdl.as_str_req()?)?))?;
				ModifierKind::Skill(skill)
			}
			"Initiative" => ModifierKind::Initiative,
			name => {
				return Err(NotInList(name.into(), vec!["Ability", "SavingThrow", "Skill"]).into());
			}
		};
		Ok(Self {
			modifier,
			context,
			kind,
		})
	}
}

impl AsKdl for AddModifier {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(self.modifier.to_string());
		match &self.kind {
			ModifierKind::Ability(ability) => {
				node.append_typed("Ability", ability.as_kdl());
			}
			ModifierKind::SavingThrow(None) => {
				node.push_entry_typed("All", "SavingThrow");
			}
			ModifierKind::SavingThrow(Some(ability)) => {
				node.append_typed("SavingThrow", ability.as_kdl());
			}
			ModifierKind::Skill(skill) => {
				node.append_typed("Skill", skill.as_kdl());
			}
			ModifierKind::Initiative => {
				node.push_entry_typed("", "Initiative");
			}
		}
		if let Some(context) = &self.context {
			node.push_entry(("context", context.clone()));
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(AddModifier);

		#[test]
		fn ability_specific_noctx() -> anyhow::Result<()> {
			let doc = "mutator \"add_modifier\" \
			\"Advantage\" (Ability)\"Specific\" \"Dexterity\"";
			let data = AddModifier {
				modifier: roll::Modifier::Advantage,
				context: None,
				kind: ModifierKind::Ability(Selector::Specific(Ability::Dexterity)),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn ability_anyof_ctx() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_modifier\" \"Advantage\" (Ability)\"AnyOf\" context=\"which use smell\" {
				|    option \"Strength\"
				|    option \"Wisdom\"
				|}
			";
			let data = AddModifier {
				modifier: roll::Modifier::Advantage,
				context: Some("which use smell".into()),
				kind: ModifierKind::Ability(Selector::AnyOf {
					id: Default::default(),
					options: vec![Ability::Strength, Ability::Wisdom],
					cannot_match: Default::default(),
					amount: 1,
				}),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn saving_throw_all() -> anyhow::Result<()> {
			let doc = "mutator \"add_modifier\" \
			\"Advantage\" (SavingThrow)\"All\" context=\"Magic\"";
			let data = AddModifier {
				modifier: roll::Modifier::Advantage,
				context: Some("Magic".into()),
				kind: ModifierKind::SavingThrow(None),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn saving_throw_any_selected() -> anyhow::Result<()> {
			let doc = "mutator \"add_modifier\" \"Advantage\" (SavingThrow)\"Any\"";
			let data = AddModifier {
				modifier: roll::Modifier::Advantage,
				context: None,
				kind: ModifierKind::SavingThrow(Some(Selector::Any {
					id: Default::default(),
					cannot_match: vec![],
				})),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn skill() -> anyhow::Result<()> {
			let doc = "mutator \"add_modifier\" \"Advantage\" \
			(Skill)\"Specific\" \"Perception\" context=\"using smell\"";
			let data = AddModifier {
				modifier: roll::Modifier::Advantage,
				context: Some("using smell".into()),
				kind: ModifierKind::Skill(Selector::Specific(Skill::Perception)),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{character::Persistent, Bundle};
		use std::path::PathBuf;

		fn character(mutator: AddModifier) -> Character {
			Character::from(Persistent {
				bundles: vec![Bundle {
					name: "TestMutator".into(),
					mutators: vec![mutator.into()],
					..Default::default()
				}
				.into()],
				..Default::default()
			})
		}

		#[test]
		fn ability_specific() {
			let character = character(AddModifier {
				modifier: roll::Modifier::Advantage,
				context: None,
				kind: ModifierKind::Ability(Selector::Specific(Ability::Dexterity)),
			});
			let modifiers = character
				.skills()
				.ability_modifiers(Ability::Dexterity)
				.get(roll::Modifier::Advantage);
			assert_eq!(
				*modifiers,
				vec![(None, PathBuf::from("TestMutator")).into()]
			);
		}

		#[test]
		fn skill_specific() {
			let character = character(AddModifier {
				modifier: roll::Modifier::Disadvantage,
				context: None,
				kind: ModifierKind::Skill(Selector::Specific(Skill::Deception)),
			});
			let modifiers = character
				.skills()
				.skill_modifiers(Skill::Deception)
				.get(roll::Modifier::Disadvantage);
			assert_eq!(
				*modifiers,
				vec![(None, PathBuf::from("TestMutator")).into()]
			);
		}

		#[test]
		fn saving_throw_all() {
			let character = character(AddModifier {
				modifier: roll::Modifier::Advantage,
				context: Some("Poison".into()),
				kind: ModifierKind::SavingThrow(None),
			});
			let modifiers = character
				.saving_throws()
				.general_modifiers()
				.get(roll::Modifier::Advantage);
			assert_eq!(
				*modifiers,
				vec![(Some("Poison".into()), PathBuf::from("TestMutator")).into()]
			);
		}

		#[test]
		fn saving_throw_specific() {
			let character = character(AddModifier {
				modifier: roll::Modifier::Advantage,
				context: Some("Poison".into()),
				kind: ModifierKind::SavingThrow(Some(Selector::Specific(Ability::Constitution))),
			});
			let modifiers = character
				.saving_throws()
				.ability_modifiers(Ability::Constitution)
				.get(roll::Modifier::Advantage);
			assert_eq!(
				*modifiers,
				vec![(Some("Poison".into()), PathBuf::from("TestMutator")).into()]
			);
		}
	}
}
