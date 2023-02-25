use crate::system::dnd5e::data::{character::Character, Ability};

#[derive(Clone, PartialEq)]
pub struct ArmorClass {
	formulas: Vec<ArmorClassFormula>,
	bonuses: Vec<i32>,
}
impl Default for ArmorClass {
	fn default() -> Self {
		Self {
			formulas: vec![ArmorClassFormula::default()],
			bonuses: Vec::new(),
		}
	}
}
impl ArmorClass {
	pub fn push_formula(&mut self, formula: ArmorClassFormula) {
		self.formulas.push(formula);
	}

	pub fn push_bonus(&mut self, bonus: i32) {
		self.bonuses.push(bonus);
	}

	pub fn evaluate(&self, state: &Character) -> i32 {
		let best_formula_value = self
			.formulas
			.iter()
			.map(|formula| formula.evaluate(state))
			.max()
			.unwrap_or(0);
		best_formula_value + self.bonuses.iter().sum::<i32>()
	}
}

#[derive(Clone, PartialEq)]
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

#[derive(Clone, Copy, PartialEq)]
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
	fn evaluate(&self, state: &Character) -> i32 {
		let value = state.ability_score(self.ability).0.modifier();
		let value = self.min.map(|min| value.max(min)).unwrap_or(value);
		let value = self.max.map(|max| value.min(max)).unwrap_or(value);
		value
	}
}
