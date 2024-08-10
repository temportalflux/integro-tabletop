use crate::system::dnd5e::data::roll::{Die, Roll, RollSet};
use enum_map::EnumMap;
use std::path::PathBuf;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct HitDice {
	roll_set: RollSet,
	rolls_with_source: Vec<(Roll, PathBuf)>,
}

impl HitDice {
	pub fn push(&mut self, roll: Roll, source: PathBuf) {
		self.roll_set.push(roll);
		self.rolls_with_source.push((roll, source));
	}

	pub fn dice(&self) -> &EnumMap<Die, i32> {
		self.roll_set.die_map()
	}

	pub fn sources(&self) -> &Vec<(Roll, PathBuf)> {
		&self.rolls_with_source
	}
}
