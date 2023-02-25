use crate::system::dnd5e::{
	data::{character::Character, roll::Roll},
	mutator::Mutator,
};

#[derive(Clone)]
pub struct AddLifeExpectancy(pub i32);
impl Mutator for AddLifeExpectancy {
	fn node_id(&self) -> &'static str {
		"add_life_expectancy"
	}

	fn apply<'c>(&self, stats: &mut Character) {
		stats.derived_description_mut().life_expectancy += self.0;
	}
}

#[derive(Clone)]
pub enum AddMaxHeight {
	Value(i32),
	Roll(Roll),
}

impl Mutator for AddMaxHeight {
	fn node_id(&self) -> &'static str {
		"add_max_height"
	}

	fn apply<'c>(&self, stats: &mut Character) {
		match self {
			Self::Value(value) => {
				stats.derived_description_mut().max_height.0 += *value;
			}
			Self::Roll(roll) => {
				stats.derived_description_mut().max_height.1.push(*roll);
			}
		}
	}
}
