use enumset::EnumSetType;

#[derive(EnumSetType, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Copy)]
pub enum ProficiencyLevel {
	None,
	Half,
	Full,
	Double,
}

impl ProficiencyLevel {
	pub fn as_display_name(&self) -> &'static str {
		match self {
			Self::None => "Not Proficient",
			Self::Half => "Half Proficient",
			Self::Full => "Proficient",
			Self::Double => "Expertise",
		}
	}

	pub fn bonus_multiplier(&self) -> f32 {
		match self {
			Self::None => 0.0,
			Self::Half => 0.5,
			Self::Full => 1.0,
			Self::Double => 2.0,
		}
	}
}
impl Into<yew::prelude::Html> for ProficiencyLevel {
	fn into(self) -> yew::prelude::Html {
		use yew::prelude::*;
		match self {
			Self::None => html! { <i class="fa-regular fa-circle" /> },
			Self::Half => {
				html! { <i class="fa-solid fa-circle-half-stroke" style="color: var(--theme-frame-color);" /> }
			}
			Self::Full => {
				html! { <i class="fa-solid fa-circle" style="color: var(--theme-frame-color);" /> }
			}
			Self::Double => {
				html! { <i class="fa-regular fa-circle-dot" style="color: var(--theme-frame-color);" /> }
			}
		}
	}
}

impl std::ops::Mul<i32> for ProficiencyLevel {
	type Output = i32;

	fn mul(self, prof_bonus: i32) -> Self::Output {
		let modified = (prof_bonus as f32) * self.bonus_multiplier();
		modified.floor() as i32
	}
}

#[derive(EnumSetType)]
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
