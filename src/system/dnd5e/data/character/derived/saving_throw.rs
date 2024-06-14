use crate::system::{
	dnd5e::data::{
		proficiency,
		roll::{Modifier, ModifierList, NumbericalBonusList},
		Ability,
	},
	mutator::ReferencePath,
};
use enum_map::EnumMap;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct SavingThrows(EnumMap<Ability, SavingThrow>);

#[derive(Clone, Default, PartialEq, Debug)]
pub struct SavingThrow(proficiency::List, ModifierList, NumbericalBonusList);

impl SavingThrow {
	fn proficiencies_mut(&mut self) -> &mut proficiency::List {
		&mut self.0
	}

	fn modifiers_mut(&mut self) -> &mut ModifierList {
		&mut self.1
	}

	fn bonuses_mut(&mut self) -> &mut NumbericalBonusList {
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

impl SavingThrows {
	fn iter_mut(&mut self, ability: Option<Ability>) -> impl Iterator<Item = (Ability, &mut SavingThrow)> {
		self.0
			.iter_mut()
			.filter(move |(key, _)| ability.is_none() || Some(*key) == ability)
	}

	pub fn iter(&self) -> impl Iterator<Item = (Ability, &SavingThrow)> {
		self.0.iter()
	}

	pub fn add_proficiency(&mut self, ability: Option<Ability>, source: &ReferencePath) {
		for (_ability, saving_throw) in self.iter_mut(ability) {
			saving_throw
				.proficiencies_mut()
				.push(proficiency::Level::Full, source.display.clone());
		}
	}

	pub fn add_modifier(
		&mut self,
		ability: Option<Ability>,
		modifier: Modifier,
		target: Option<String>,
		source: &ReferencePath,
	) {
		for (_ability, saving_throw) in self.iter_mut(ability) {
			saving_throw
				.modifiers_mut()
				.push(modifier, target.clone(), source.display.clone());
		}
	}

	pub fn add_bonus(&mut self, ability: Option<Ability>, bonus: i64, target: Option<String>, source: &ReferencePath) {
		for (_ability, saving_throw) in self.iter_mut(ability) {
			saving_throw
				.bonuses_mut()
				.push(bonus, target.clone(), source.display.clone());
		}
	}
}
impl std::ops::Index<Ability> for SavingThrows {
	type Output = SavingThrow;
	fn index(&self, index: Ability) -> &Self::Output {
		&self.0[index]
	}
}
