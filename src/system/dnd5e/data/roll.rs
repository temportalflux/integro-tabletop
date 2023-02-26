use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Roll {
	pub amount: i32,
	pub die: Die,
}
impl ToString for Roll {
	fn to_string(&self) -> String {
		format!(
			"{}d{}",
			self.amount,
			match self.die {
				Die::D4 => 4,
				Die::D6 => 6,
				Die::D8 => 8,
				Die::D10 => 10,
				Die::D12 => 12,
				Die::D20 => 20,
			}
		)
	}
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

#[derive(Debug, Enum, EnumSetType, PartialOrd, Ord)]
pub enum Modifier {
	Advantage,
	Disadvantage,
}
impl Modifier {
	pub fn display_name(&self) -> &'static str {
		match self {
			Modifier::Advantage => "Advantage",
			Modifier::Disadvantage => "Disadvantage",
		}
	}
}
