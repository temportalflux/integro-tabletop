use crate::GeneralError;
use enum_map::Enum;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Debug, Enum, EnumSetType, PartialOrd, Ord, Hash)]
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
impl ToString for Modifier {
	fn to_string(&self) -> String {
		self.display_name().to_owned()
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
