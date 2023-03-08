use crate::{
	system::dnd5e::data::{character::Character, item::armor, Ability, Skill, WeaponProficiency},
	utility::Evaluator,
};

#[derive(Clone, PartialEq, Debug)]
pub enum IsProficientWith {
	SavingThrow(Ability),
	Skill(Skill),
	Language(String),
	Armor(armor::Kind),
	Weapon(WeaponProficiency),
	Tool(String),
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
			_ => unimplemented!(),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod evaluate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			mutator::AddProficiency,
			Feature,
		};

		fn character_with_profs(mutators: Vec<AddProficiency>) -> Character {
			let mut persistent = Persistent::default();
			persistent.feats.push(
				Feature {
					name: "CustomFeat".into(),
					mutators: mutators.into_iter().map(|v| v.into()).collect(),
					..Default::default()
				}
				.into(),
			);
			Character::from(persistent)
		}
	}
}
