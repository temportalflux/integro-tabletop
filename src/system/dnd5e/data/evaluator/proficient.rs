use crate::{
	system::dnd5e::data::{character::Character, WeaponProficiency},
	utility::Evaluator,
};

#[derive(Clone, PartialEq, Debug)]
pub enum IsProficientWith {
	Weapon(WeaponProficiency),
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
