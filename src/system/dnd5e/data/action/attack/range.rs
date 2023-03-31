use crate::utility::InvalidEnumStr;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Debug, EnumSetType)]
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
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Self" => Ok(Self::OnlySelf),
			"Touch" => Ok(Self::Touch),
			"Bounded" => Ok(Self::Bounded),
			"Sight" => Ok(Self::Sight),
			"Unlimited" => Ok(Self::Unlimited),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}
