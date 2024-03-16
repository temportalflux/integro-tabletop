use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::data::{character::Character, description, ArmorClassFormula},
	system::Mutator,
};
use kdlize::{AsKdl, FromKdl};

#[derive(Clone, PartialEq, Debug)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);

crate::impl_trait_eq!(AddArmorClassFormula);
kdlize::impl_kdl_node!(AddArmorClassFormula, "add_armor_class_formula");

impl Mutator for AddArmorClassFormula {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
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
			args.push(format!("your {} modifier{}", bonus.ability.long_name(), bounds));
		}
		description::Section {
			title: Some("Armor Class".into()),
			content: format!("You can calculate your Armor Class using {}.", args.join(" + ")).into(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		stats.armor_class_mut().push_formula(self.0.clone(), parent);
	}
}

impl FromKdl<NodeContext> for AddArmorClassFormula {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(Self(ArmorClassFormula::from_kdl(node)?))
	}
}

impl AsKdl for AddArmorClassFormula {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		self.0.as_kdl()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::dnd5e::{
				data::{Ability, BoundedAbility},
				mutator::test::test_utils,
			},
		};

		test_utils!(AddArmorClassFormula);

		#[test]
		fn base_only() -> anyhow::Result<()> {
			let doc = "mutator \"add_armor_class_formula\" base=12";
			let data = AddArmorClassFormula(ArmorClassFormula {
				base: 12,
				bonuses: vec![],
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn one_bonus_unbounded() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_armor_class_formula\" base=12 {
				|    bonus (Ability)\"Dexterity\"
				|}
			";
			let data = AddArmorClassFormula(ArmorClassFormula {
				base: 12,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: None,
				}],
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn one_bonus_bounded() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_armor_class_formula\" base=15 {
				|    bonus (Ability)\"Dexterity\" max=2
				|}
			";
			let data = AddArmorClassFormula(ArmorClassFormula {
				base: 15,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: Some(2),
				}],
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn multiple_bonus() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_armor_class_formula\" base=10 {
				|    bonus (Ability)\"Dexterity\"
				|    bonus (Ability)\"Wisdom\"
				|}
			";
			let data = AddArmorClassFormula(ArmorClassFormula {
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
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			Ability, ArmorClassFormula, Bundle,
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
				bundles: vec![Bundle {
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
