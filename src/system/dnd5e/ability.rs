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

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Score(pub i32);
impl std::ops::Deref for Score {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Score {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Score {
	pub fn modifier(&self) -> i32 {
		(((self.0 - 10) as f32) / 2f32).floor() as i32
	}
}
