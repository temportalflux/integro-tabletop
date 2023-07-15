use crate::GeneralError;
use enum_map::Enum;
use enumset::EnumSetType;
use std::str::FromStr;

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

	pub fn roll(&self, rand: &mut impl rand::Rng, num: u32) -> u32 {
		if num == 0 {
			return 0;
		}
		let range = 1..=self.value();
		(0..num).map(|_| rand.gen_range(range.clone())).sum()
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
impl std::fmt::Display for Die {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::D4 => write!(f, "d4"),
			Self::D6 => write!(f, "d6"),
			Self::D8 => write!(f, "d8"),
			Self::D10 => write!(f, "d10"),
			Self::D12 => write!(f, "d12"),
			Self::D20 => write!(f, "d20"),
		}
	}
}
