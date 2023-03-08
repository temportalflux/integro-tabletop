use crate::{
	system::dnd5e::{
		data::{character::Character, ArmorClassFormula},
		KDLNode,
	},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);

impl crate::utility::TraitEq for AddArmorClassFormula {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for AddArmorClassFormula {
	fn id() -> &'static str {
		"add_armor_class_formula"
	}
}

impl Mutator for AddArmorClassFormula {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.armor_class_mut().push_formula(self.0.clone(), source);
	}
}

// TODO: FromKDL ArmorClassFormula
// TODO: Test FromKDL ArmorClassFormula

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
