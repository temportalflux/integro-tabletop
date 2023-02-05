use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Roll {
	pub amount: i32,
	pub die: Die,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct RollSet(pub EnumMap<Die, i32>);

impl RollSet {
	pub fn push(&mut self, roll: Roll) {
		self.0[roll.die] += roll.amount;
	}
}

#[derive(Debug, Enum, EnumSetType)]
pub enum Die {
	D4,
	D6,
	D8,
	D10,
	D12,
	D20,
}
