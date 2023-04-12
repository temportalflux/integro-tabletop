use crate::utility::InvalidEnumStr;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Debug, EnumSetType)]
pub enum Rarity {
	Common,
	Uncommon,
	Rare,
	VeryRare,
	Legendary,
}

impl ToString for Rarity {
	fn to_string(&self) -> String {
		match self {
			Self::Common => "Common",
			Self::Uncommon => "Uncommon",
			Self::Rare => "Rare",
			Self::VeryRare => "VeryRare",
			Self::Legendary => "Legendary",
		}
		.to_owned()
	}
}

impl FromStr for Rarity {
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Common" => Ok(Self::Common),
			"Uncommon" => Ok(Self::Uncommon),
			"Rare" => Ok(Self::Rare),
			"VeryRare" => Ok(Self::VeryRare),
			"Legendary" => Ok(Self::Legendary),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}
