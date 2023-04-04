use crate::utility::InvalidEnumStr;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Default, Debug, EnumSetType, PartialOrd, Ord, Hash)]
pub enum Kind {
	#[default]
	Simple,
	Martial,
}

impl ToString for Kind {
	fn to_string(&self) -> String {
		match self {
			Self::Simple => "Simple",
			Self::Martial => "Martial",
		}
		.to_owned()
	}
}

impl FromStr for Kind {
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Simple" => Ok(Self::Simple),
			"Martial" => Ok(Self::Martial),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}
