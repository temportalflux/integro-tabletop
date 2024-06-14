use crate::system::dnd5e::data::{
	proficiency,
	roll::{ModifierList, NumbericalBonusList},
};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Initiative(proficiency::List, ModifierList, NumbericalBonusList);

impl Initiative {
	pub fn proficiencies_mut(&mut self) -> &mut proficiency::List {
		&mut self.0
	}

	pub fn modifiers_mut(&mut self) -> &mut ModifierList {
		&mut self.1
	}

	pub fn bonuses_mut(&mut self) -> &mut NumbericalBonusList {
		&mut self.2
	}

	pub fn proficiencies(&self) -> &proficiency::List {
		&self.0
	}

	pub fn modifiers(&self) -> &ModifierList {
		&self.1
	}

	pub fn bonuses(&self) -> &NumbericalBonusList {
		&self.2
	}
}
