use crate::{
	system::dnd5e::data::{character::Character, WeaponProficiency},
	utility::Evaluator,
};

#[derive(Clone, PartialEq, Debug)]
pub enum IsProficientWith {
	Weapon(WeaponProficiency),
}

impl crate::utility::TraitEq for IsProficientWith {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for IsProficientWith {
	type Context = Character;
	type Item = bool;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		match self {
			Self::Weapon(proficiency) => state
				.other_proficiencies()
				.weapons
				.contains_key(proficiency),
		}
	}
}
