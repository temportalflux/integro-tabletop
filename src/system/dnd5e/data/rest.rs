use crate::utility::InvalidEnumStr;
use enum_map::Enum;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Debug, EnumSetType, Enum)]
pub enum Rest {
	Short,
	Long,
}

impl ToString for Rest {
	fn to_string(&self) -> String {
		match self {
			Self::Short => "Short",
			Self::Long => "Long",
		}
		.to_owned()
	}
}

impl FromStr for Rest {
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Short" => Ok(Self::Short),
			"Long" => Ok(Self::Long),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}
