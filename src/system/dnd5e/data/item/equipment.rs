use super::{armor::Armor, weapon::Weapon};
use crate::{
	system::dnd5e::{data::character::Character, BoxedCriteria, BoxedMutator},
	utility::MutatorGroup,
};

#[derive(Clone, PartialEq, Default)]
pub struct Equipment {
	/// The criteria which must be met for this item to be equipped.
	pub criteria: Option<BoxedCriteria>,
	/// Passive modifiers applied while this item is equipped.
	pub modifiers: Vec<BoxedMutator>,
	/// If this item is armor, this is the armor data.
	pub armor: Option<Armor>,
	/// If this item is a shield, this is the AC bonus it grants.
	pub shield: Option<i32>,
	/// If this item is a weapon, tthis is the weapon data.
	pub weapon: Option<Weapon>,
	/// If this weapon can be attuned, this is the attunement data.
	pub attunement: Option<Attunement>,
}

impl MutatorGroup for Equipment {
	type Target = Character;

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		for modifier in &self.modifiers {
			stats.apply(modifier);
		}
		if let Some(armor) = &self.armor {
			stats.apply_from(armor);
		}
		if let Some(shield) = &self.shield {
			let source = stats.source_path();
			stats.armor_class_mut().push_bonus(*shield, source);
		}
	}
}

impl Equipment {
	/// Returs Ok if the item can currently be equipped, otherwise returns a user-displayable reason why it cannot be equipped.
	pub fn can_be_equipped(&self, state: &Character) -> Result<(), String> {
		match &self.criteria {
			Some(criteria) => state.evaluate(criteria),
			None => Ok(()),
		}
	}
}

#[derive(Clone, PartialEq, Default)]
pub struct Attunement {
	pub modifiers: Vec<BoxedMutator>,
}
