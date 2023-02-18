use crate::system::dnd5e::{character::DerivedBuilder, roll::Roll};

#[derive(Clone)]
pub struct AddLifeExpectancy(pub i32);
impl super::Mutator for AddLifeExpectancy {
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		stats.life_expectancy += self.0;
	}
}

#[derive(Clone)]
pub enum AddMaxHeight {
	Value(i32),
	Roll(Roll),
}

impl super::Mutator for AddMaxHeight {
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		match self {
			Self::Value(value) => {
				stats.max_height.0 += *value;
			}
			Self::Roll(roll) => {
				stats.max_height.1.push(*roll);
			}
		}
	}
}
