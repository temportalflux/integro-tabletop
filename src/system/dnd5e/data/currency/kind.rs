use crate::utility::InvalidEnumStr;
use enum_map::Enum;
use enumset::{EnumSet, EnumSetType};
use std::str::FromStr;

#[derive(Debug, EnumSetType, Enum, PartialOrd, Ord, Hash)]
pub enum CurrencyKind {
	Copper,
	Silver,
	Electrum,
	Gold,
	Platinum,
}

impl CurrencyKind {
	pub fn all() -> impl Iterator<Item = Self> {
		EnumSet::all().into_iter()
	}

	pub fn multiplier(&self) -> u64 {
		match self {
			Self::Copper => 1,
			Self::Silver => 10,
			Self::Electrum => 50,
			Self::Gold => 100,
			Self::Platinum => 1000,
		}
	}

	pub fn abbreviation(&self) -> &'static str {
		match self {
			Self::Copper => "cp",
			Self::Silver => "sp",
			Self::Electrum => "ep",
			Self::Gold => "gp",
			Self::Platinum => "pp",
		}
	}
}

impl ToString for CurrencyKind {
	fn to_string(&self) -> String {
		match self {
			Self::Copper => "Copper",
			Self::Silver => "Silver",
			Self::Electrum => "Electrum",
			Self::Gold => "Gold",
			Self::Platinum => "Platinum",
		}
		.to_owned()
	}
}

impl FromStr for CurrencyKind {
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Copper" => Ok(Self::Copper),
			"Silver" => Ok(Self::Silver),
			"Electrum" => Ok(Self::Electrum),
			"Gold" => Ok(Self::Gold),
			"Platinum" => Ok(Self::Platinum),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}
