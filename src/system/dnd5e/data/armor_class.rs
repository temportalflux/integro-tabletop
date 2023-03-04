use std::path::PathBuf;

use crate::system::dnd5e::data::{character::Character, Ability};

#[derive(Clone, PartialEq)]
pub struct ArmorClass {
	formulas: Vec<(ArmorClassFormula, PathBuf)>,
	bonuses: Vec<(i32, PathBuf)>,
}
impl Default for ArmorClass {
	fn default() -> Self {
		Self {
			formulas: vec![(ArmorClassFormula::default(), PathBuf::new())],
			bonuses: Vec::new(),
		}
	}
}
impl ArmorClass {
	pub fn push_formula(&mut self, formula: ArmorClassFormula, source: PathBuf) {
		self.formulas.push((formula, source));
	}

	pub fn push_bonus(&mut self, bonus: i32, source: PathBuf) {
		self.bonuses.push((bonus, source));
	}

	pub fn evaluate(&self, state: &Character) -> i32 {
		let best_formula_value = self
			.formulas
			.iter()
			.map(|(formula, _)| formula.evaluate(state))
			.max()
			.unwrap_or(0);
		best_formula_value + self.bonuses.iter().map(|(value, _)| value).sum::<i32>()
	}

	pub fn iter_formulas(&self) -> impl Iterator<Item = &(ArmorClassFormula, PathBuf)> {
		self.formulas.iter()
	}

	pub fn iter_bonuses(&self) -> impl Iterator<Item = &(i32, PathBuf)> {
		self.bonuses.iter()
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorClassFormula {
	pub base: u32,
	pub bonuses: Vec<BoundedAbility>,
}
impl Default for ArmorClassFormula {
	fn default() -> Self {
		Self {
			base: 10,
			bonuses: vec![BoundedAbility {
				ability: Ability::Dexterity,
				min: None,
				max: None,
			}],
		}
	}
}
impl From<u32> for ArmorClassFormula {
	fn from(base: u32) -> Self {
		Self {
			base,
			bonuses: Vec::new(),
		}
	}
}
impl ArmorClassFormula {
	fn evaluate(&self, state: &Character) -> i32 {
		let bonus: i32 = self
			.bonuses
			.iter()
			.map(|bounded| bounded.evaluate(state))
			.sum();
		(self.base as i32) + bonus
	}
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundedAbility {
	pub ability: Ability,
	pub min: Option<i32>,
	pub max: Option<i32>,
}
impl From<Ability> for BoundedAbility {
	fn from(ability: Ability) -> Self {
		Self {
			ability,
			min: None,
			max: None,
		}
	}
}
impl BoundedAbility {
	pub fn evaluate(&self, state: &Character) -> i32 {
		let value = state.ability_modifier(self.ability, None);
		let value = self.min.map(|min| value.max(min)).unwrap_or(value);
		let value = self.max.map(|max| value.min(max)).unwrap_or(value);
		value
	}
}
