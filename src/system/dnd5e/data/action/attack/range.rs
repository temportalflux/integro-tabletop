use std::str::FromStr;

use crate::GeneralError;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RangeKind {
	OnlySelf,
	Touch,
	Bounded, // abide by the short/long dist range
	Sight,
	Unlimited,
}

impl ToString for RangeKind {
	fn to_string(&self) -> String {
		match self {
			Self::OnlySelf => "Self",
			Self::Touch => "Touch",
			Self::Bounded => "Bounded",
			Self::Sight => "Sight",
			Self::Unlimited => "Unlimited",
		}
		.to_owned()
	}
}

impl FromStr for RangeKind {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Self" => Ok(Self::OnlySelf),
			"Touch" => Ok(Self::Touch),
			"Bounded" => Ok(Self::Bounded),
			"Sight" => Ok(Self::Sight),
			"Unlimited" => Ok(Self::Unlimited),
			_ => Err(GeneralError(format!(
				"Invalid kind of range {s:?}, expected Self, Touch, Bounded, Sight, or Unlimited."
			))),
		}
	}
}
