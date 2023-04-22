use crate::{
	kdl_ext::FromKDL,
	system::dnd5e::data::{character::Character, ArmorClassFormula},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);

crate::impl_trait_eq!(AddArmorClassFormula);
crate::impl_kdl_node!(AddArmorClassFormula, "add_armor_class_formula");

impl Mutator for AddArmorClassFormula {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some("Armor Class".into())
	}

	fn description(&self) -> Option<String> {
		let mut args = Vec::new();
		if self.0.base > 0 {
			args.push(format!("{}", self.0.base));
		}
		for bonus in &self.0.bonuses {
			let bounds = match (bonus.min, bonus.max) {
				(None, None) => String::new(),
				(Some(min), None) => format!(" (min {min:+})"),
				(None, Some(max)) => format!(" (max {max:+})"),
				(Some(min), Some(max)) => format!(" (min {min:+}, max {max:+})"),
			};
			args.push(format!(
				"your {} modifier{}",
				bonus.ability.long_name(),
				bounds
			));
		}
		Some(format!(
			"You can calculate your Armor Class using {}.",
			args.join(" + ")
		))
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		stats
			.armor_class_mut()
			.push_formula(self.0.clone(), parent.to_owned());
	}
}

impl FromKDL for AddArmorClassFormula {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(ArmorClassFormula::from_kdl(node, ctx)?))
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::{
			core::NodeRegistry,
			dnd5e::{
				data::{Ability, BoundedAbility},
				BoxedMutator,
			},
		};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<AddArmorClassFormula>(doc)
		}

		#[test]
		fn base_only() -> anyhow::Result<()> {
			let doc = "mutator \"add_armor_class_formula\" base=12";
			let expected = AddArmorClassFormula(ArmorClassFormula {
				base: 12,
				bonuses: vec![],
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn one_bonus_unbounded() -> anyhow::Result<()> {
			let doc = "mutator \"add_armor_class_formula\" base=12 {
				bonus \"Dexterity\"
			}";
			let expected = AddArmorClassFormula(ArmorClassFormula {
				base: 12,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: None,
				}],
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn one_bonus_bounded() -> anyhow::Result<()> {
			let doc = "mutator \"add_armor_class_formula\" base=15 {
				bonus \"Dexterity\" max=2
			}";
			let expected = AddArmorClassFormula(ArmorClassFormula {
				base: 15,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: Some(2),
				}],
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn multiple_bonus() -> anyhow::Result<()> {
			let doc = "mutator \"add_armor_class_formula\" base=10 {
				bonus \"Dexterity\"
				bonus \"Wisdom\"
			}";
			let expected = AddArmorClassFormula(ArmorClassFormula {
				base: 10,
				bonuses: vec![
					BoundedAbility {
						ability: Ability::Dexterity,
						min: None,
						max: None,
					},
					BoundedAbility {
						ability: Ability::Wisdom,
						min: None,
						max: None,
					},
				],
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			Ability, ArmorClassFormula, Feature,
		};

		#[test]
		fn no_formula() {
			let character = Character::from(Persistent {
				ability_scores: enum_map::enum_map! {
					Ability::Strength => 10,
					Ability::Dexterity => 12,
					Ability::Constitution => 15,
					Ability::Intelligence => 10,
					Ability::Wisdom => 10,
					Ability::Charisma => 10,
				},
				..Default::default()
			});
			assert_eq!(character.armor_class().evaluate(&character), 11);
		}

		#[test]
		fn with_modifier() {
			let character = Character::from(Persistent {
				ability_scores: enum_map::enum_map! {
					Ability::Strength => 10,
					Ability::Dexterity => 12,
					Ability::Constitution => 15,
					Ability::Intelligence => 10,
					Ability::Wisdom => 10,
					Ability::Charisma => 10,
				},
				feats: vec![Feature {
					mutators: vec![AddArmorClassFormula(ArmorClassFormula {
						base: 11,
						bonuses: vec![Ability::Dexterity.into(), Ability::Constitution.into()],
					})
					.into()],
					..Default::default()
				}
				.into()],
				..Default::default()
			});
			// Max of:
			// 10 + Dex (ArmorClassFormula::default())
			// 11 + Dex + Con
			assert_eq!(character.armor_class().evaluate(&character), 14);
		}
	}
}