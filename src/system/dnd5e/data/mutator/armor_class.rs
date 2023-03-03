use crate::{
	system::dnd5e::data::{character::Character, ArmorClassFormula},
	utility::Mutator,
};

#[derive(Clone, PartialEq)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);
impl Mutator for AddArmorClassFormula {
	type Target = Character;

	fn node_name() -> &'static str {
		"add_armor_class_formula"
	}

	fn get_node_name(&self) -> &'static str {
		Self::node_name()
	}

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
