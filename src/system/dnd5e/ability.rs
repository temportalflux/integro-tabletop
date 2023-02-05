use enum_map::Enum;
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Debug, EnumSetType, Enum, PartialOrd, Ord, Hash)]
pub enum Ability {
	Strength,
	Dexterity,
	Constitution,
	Intelligence,
	Wisdom,
	Charisma,
}

impl Ability {
	pub fn long_name(&self) -> &'static str {
		match self {
			Self::Strength => "Strength",
			Self::Dexterity => "Dexterity",
			Self::Constitution => "Constitution",
			Self::Intelligence => "Intelligence",
			Self::Wisdom => "Wisdom",
			Self::Charisma => "Charisma",
		}
	}

	pub fn abbreviated_name(&self) -> &'static str {
		match self {
			Self::Strength => "str",
			Self::Dexterity => "dex",
			Self::Constitution => "con",
			Self::Intelligence => "int",
			Self::Wisdom => "wis",
			Self::Charisma => "cha",
		}
	}
}

impl FromStr for Ability {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"str" | "strength" => Ok(Self::Strength),
			"dex" | "dexterity" => Ok(Self::Dexterity),
			"con" | "constitution" => Ok(Self::Constitution),
			"int" | "intelligence" => Ok(Self::Intelligence),
			"wis" | "wisdom" => Ok(Self::Wisdom),
			"cha" | "charisma" => Ok(Self::Charisma),
			_ => Err(()),
		}
	}
}
