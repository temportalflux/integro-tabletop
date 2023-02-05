use crate::system::dnd5e::{character::CompiledStats, roll::Roll, Character};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AddLifeExpectancy(pub i32);
impl super::Modifier for AddLifeExpectancy {
	fn apply(&self, _: &Character, stats: &mut CompiledStats, _: PathBuf) {
		stats.life_expectancy += self.0;
	}
}

#[derive(Clone)]
pub enum AddMaxHeight {
	Value(i32),
	Roll(Roll),
}

impl super::Modifier for AddMaxHeight {
	fn apply(&self, _: &Character, stats: &mut CompiledStats, _: PathBuf) {
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
