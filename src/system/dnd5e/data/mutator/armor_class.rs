use crate::{
	system::dnd5e::data::{character::Character, ArmorClassFormula},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);

crate::impl_trait_eq!(AddArmorClassFormula);
crate::impl_kdl_node!(AddArmorClassFormula, "add_armor_class_formula");

impl Mutator for AddArmorClassFormula {
	type Target = Character;

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.armor_class_mut().push_formula(self.0.clone(), source);
	}
}

#[cfg(test)]
mod test {
	use super::AddArmorClassFormula;
	use crate::system::dnd5e::data::{
		character::{Character, Persistent},
		Ability, ArmorClassFormula, Feature, Score,
	};

	#[test]
	fn no_formula() {
		let character = Character::from(Persistent {
			ability_scores: enum_map::enum_map! {
				Ability::Strength => Score(10),
				Ability::Dexterity => Score(12),
				Ability::Constitution => Score(15),
				Ability::Intelligence => Score(10),
				Ability::Wisdom => Score(10),
				Ability::Charisma => Score(10),
			},
			..Default::default()
		});
		assert_eq!(character.armor_class().evaluate(&character), 11);
	}

	#[test]
	fn with_modifier() {
		let character = Character::from(Persistent {
			ability_scores: enum_map::enum_map! {
				Ability::Strength => Score(10),
				Ability::Dexterity => Score(12),
				Ability::Constitution => Score(15),
				Ability::Intelligence => Score(10),
				Ability::Wisdom => Score(10),
				Ability::Charisma => Score(10),
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
