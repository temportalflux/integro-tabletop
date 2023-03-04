use crate::{
	system::dnd5e::{
		data::{character::Character, roll::Roll},
		KDLNode,
	},
	utility::Mutator,
};

#[derive(Clone, Debug)]
pub struct AddLifeExpectancy(pub i32);
impl KDLNode for AddLifeExpectancy {
	fn id() -> &'static str {
		"extend_life_expectancy"
	}
}
impl Mutator for AddLifeExpectancy {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		stats.derived_description_mut().life_expectancy += self.0;
	}
}

#[derive(Clone, Debug)]
pub enum AddMaxHeight {
	Value(i32),
	Roll(Roll),
}

impl KDLNode for AddMaxHeight {
	fn id() -> &'static str {
		"add_max_height"
	}
}

impl Mutator for AddMaxHeight {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
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
