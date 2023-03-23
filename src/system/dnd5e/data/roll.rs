use std::str::FromStr;

use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;

use crate::GeneralError;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
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
impl FromStr for Roll {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		static EXPECTED: &'static str = "{int}d{int}";
		if !s.contains('d') {
			return Err(GeneralError(format!(
				"Invalid Roll string format. Expected {EXPECTED:?}, but found {s}"
			))
			.into());
		}
		let mut parts = s.split('d');
		let amount_str = parts.next().ok_or(GeneralError(format!(
			"Roll string {s:?} missing amount, expected format {EXPECTED:?}."
		)))?;
		let die_str = parts.next().ok_or(GeneralError(format!(
			"Roll string {s:?} missing die type, expected format {EXPECTED:?}."
		)))?;
		if parts.next().is_some() {
			return Err(GeneralError(format!(
				"Too many parts in {s:?} for Roll, expected {EXPECTED:?}"
			))
			.into());
		}
		let amount = amount_str.parse::<u32>()? as i32;
		let die = Die::try_from(die_str.parse::<u32>()?)?;
		Ok(Self { amount, die })
	}
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct RollSet(pub EnumMap<Die, i32>);

impl RollSet {
	pub fn push(&mut self, roll: Roll) {
		self.0[roll.die] += roll.amount;
	}
}

#[derive(Debug, Enum, EnumSetType, Default)]
pub enum Die {
	#[default]
	D4,
	D6,
	D8,
	D10,
	D12,
	D20,
}
impl Die {
	pub fn value(self) -> u32 {
		match self {
			Self::D4 => 4,
			Self::D6 => 6,
			Self::D8 => 8,
			Self::D10 => 10,
			Self::D12 => 12,
			Self::D20 => 20,
		}
	}
}
impl TryFrom<u32> for Die {
	type Error = GeneralError;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		match value {
			4 => Ok(Self::D4),
			6 => Ok(Self::D6),
			8 => Ok(Self::D8),
			10 => Ok(Self::D10),
			12 => Ok(Self::D12),
			20 => Ok(Self::D20),
			_ => Err(GeneralError(format!("Invalid die number: {value}"))),
		}
	}
}
impl FromStr for Die {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"d4" => Ok(Self::D4),
			"d6" => Ok(Self::D6),
			"d8" => Ok(Self::D8),
			"d10" => Ok(Self::D10),
			"d12" => Ok(Self::D12),
			"d20" => Ok(Self::D20),
			_ => Err(GeneralError(format!(
				"Invalid die type {s:?}, expected d4, d6, d8, d10, d12, or d20"
			))),
		}
	}
}
impl ToString for Die {
	fn to_string(&self) -> String {
		match self {
			Self::D4 => "d4",
			Self::D6 => "d6",
			Self::D8 => "d8",
			Self::D10 => "d10",
			Self::D12 => "d12",
			Self::D20 => "d20",
		}
		.to_owned()
	}
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
impl FromStr for Modifier {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Advantage" => Ok(Self::Advantage),
			"Disadvantage" => Ok(Self::Disadvantage),
			_ => Err(GeneralError(format!(
				"Invalid roll modifier value {s:?}, expected Advantage or Disadvantage."
			))),
		}
	}
}
