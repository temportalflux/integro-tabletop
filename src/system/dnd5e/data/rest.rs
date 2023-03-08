use crate::GeneralError;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Debug)]
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
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Short" => Ok(Self::Short),
			"Long" => Ok(Self::Long),
			_ => Err(GeneralError(format!(
				"Invalid rest {s:?}, expected Short or Long"
			))),
		}
	}
}
