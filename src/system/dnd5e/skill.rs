use std::str::FromStr;

use super::Ability;
use enum_map::Enum;
use enumset::EnumSetType;
use serde::{Deserialize, Serialize};

#[derive(EnumSetType, PartialOrd, Enum, Serialize, Deserialize, Debug)]
pub enum Skill {
	Acrobatics,
	AnimalHandling,
	Arcana,
	Athletics,
	Deception,
	History,
	Insight,
	Intimidation,
	Investigation,
	Medicine,
	Nature,
	Perception,
	Performance,
	Persuasion,
	Religion,
	SleightOfHand,
	Stealth,
	Survival,
}

impl Skill {
	pub fn ability(&self) -> Ability {
		match self {
			Self::Acrobatics => Ability::Dexterity,
			Self::AnimalHandling => Ability::Wisdom,
			Self::Arcana => Ability::Intelligence,
			Self::Athletics => Ability::Strength,
			Self::Deception => Ability::Charisma,
			Self::History => Ability::Intelligence,
			Self::Insight => Ability::Wisdom,
			Self::Intimidation => Ability::Charisma,
			Self::Investigation => Ability::Intelligence,
			Self::Medicine => Ability::Wisdom,
			Self::Nature => Ability::Intelligence,
			Self::Perception => Ability::Wisdom,
			Self::Performance => Ability::Charisma,
			Self::Persuasion => Ability::Charisma,
			Self::Religion => Ability::Intelligence,
			Self::SleightOfHand => Ability::Dexterity,
			Self::Stealth => Ability::Dexterity,
			Self::Survival => Ability::Wisdom,
		}
	}

	pub fn display_name(&self) -> &'static str {
		match self {
			Self::Acrobatics => "Acrobatics",
			Self::AnimalHandling => "Animal Handling",
			Self::Arcana => "Arcana",
			Self::Athletics => "Athletics",
			Self::Deception => "Deception",
			Self::History => "History",
			Self::Insight => "Insight",
			Self::Intimidation => "Intimidation",
			Self::Investigation => "Investigation",
			Self::Medicine => "Medicine",
			Self::Nature => "Nature",
			Self::Perception => "Perception",
			Self::Performance => "Performance",
			Self::Persuasion => "Persuasion",
			Self::Religion => "Religion",
			Self::SleightOfHand => "Sleight of Hand",
			Self::Stealth => "Stealth",
			Self::Survival => "Survival",
		}
	}
}

impl FromStr for Skill {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"acrobatics" => Ok(Self::Acrobatics),
			"animalhandling" => Ok(Self::AnimalHandling),
			"arcana" => Ok(Self::Arcana),
			"athletics" => Ok(Self::Athletics),
			"deception" => Ok(Self::Deception),
			"history" => Ok(Self::History),
			"insight" => Ok(Self::Insight),
			"intimidation" => Ok(Self::Intimidation),
			"investigation" => Ok(Self::Investigation),
			"medicine" => Ok(Self::Medicine),
			"nature" => Ok(Self::Nature),
			"perception" => Ok(Self::Perception),
			"performance" => Ok(Self::Performance),
			"persuasion" => Ok(Self::Persuasion),
			"religion" => Ok(Self::Religion),
			"sleightofhand" => Ok(Self::SleightOfHand),
			"stealth" => Ok(Self::Stealth),
			"survival" => Ok(Self::Survival),
			_ => Err(()),
		}
	}
}
