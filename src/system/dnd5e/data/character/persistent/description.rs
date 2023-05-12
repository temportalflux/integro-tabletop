use crate::{system::dnd5e::data::Size, utility::NotInList};
use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use std::{collections::HashSet, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Description {
	pub name: String,
	pub pronouns: HashSet<String>,
	pub custom_pronouns: String,
	pub height: u32,
	pub weight: u32,
	pub personality: EnumMap<PersonalityKind, Vec<String>>,
}

impl Description {
	pub fn size(&self) -> Size {
		match self.height {
			v if v >= 45 => Size::Medium,
			_ => Size::Small,
		}
	}
}

static DESC_TRAIT: &'static str = "Give your character two personality traits. \
Personality traits are small, simple ways to help you set your character apart from every other character. \
Your personality traits should tell you something interesting and fun about your character. \
They should be self-descriptions that are specific about what makes your character stand out. \
\"I'm smart\" is not a good trait, because it describes a lot of characters. \"I've read every book in Candlekeep\" \
tells you something specific about your character's interests and disposition.

Personality traits might describe the things your character likes, his or her past accomplishments, \
things your character dislikes or fears, your character's self-attitude or mannerisms, \
or the influence of his or her ability scores.

A useful place to start thinking about personality traits is to look at your highest and lowest \
ability scores and define one trait related to each. Either one could be positive or negative: \
you might work hard to overcome a low score, for example, or be cocky about your high score.";
static DESC_IDEAL: &'static str = "Describe one ideal that drives your character. \
Your ideals are the things that you believe in most strongly, the fundamental moral and \
ethical principles that compel you to act as you do. Ideals encompass everything from \
your life goals to your core belief system.

Ideals might answer any of these questions: What are the principles that you will never betray? \
What would prompt you to make sacrifices? What drives you to act and guides your goals and \
ambitions? What is the single most important thing you strive for?

You can choose any ideals you like, but your character's alignment is a good place to start defining them. \
Each background in this chapter includes six suggested ideals. Five of them are linked to aspects of alignment: \
law, chaos, good, evil, and neutrality. The last one has more to do with the \
particular background than with moral or ethical perspectives.";
static DESC_BOND: &'static str = "Create one bond for your character. \
Bonds represent a character's connections to people, places, and events in the world. \
They tie you to things from your background. They might inspire you to heights of heroism, \
or lead you to act against your own best interests if they are threatened. They can work very much like ideals, \
driving a character's motivations and goals.

Bonds might answer any of these questions: Whom do you care most about? \
To what place do you feel a special connection? What is your most treasured possession?

Your bonds might be tied to your class, your background, your race, or some other aspect of \
your character's history or personality. You might also gain new bonds over the course of your adventures.";
static DESC_FLAW: &'static str = "Choose a flaw for your character. Your character's flaw represents \
some vice, compulsion, fear, or weakness—in particular, anything that someone else could exploit to \
bring you to ruin or cause you to act against your best interests. More significant than \
negative personality traits, a flaw might answer any of these questions: What enrages you? \
What's the one person, concept, or event that you are terrified of? What are your vices?";

#[derive(Debug, EnumSetType, Enum)]
pub enum PersonalityKind {
	Trait,
	Ideal,
	Bond,
	Flaw,
}

impl std::fmt::Display for PersonalityKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Trait => "Trait",
				Self::Ideal => "Ideal",
				Self::Bond => "Bond",
				Self::Flaw => "Flaw",
			}
		)
	}
}

impl PersonalityKind {
	pub fn description(&self) -> &'static str {
		match self {
			Self::Trait => DESC_TRAIT,
			Self::Ideal => DESC_IDEAL,
			Self::Bond => DESC_BOND,
			Self::Flaw => DESC_FLAW,
		}
	}
}

impl FromStr for PersonalityKind {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Trait" => Ok(Self::Trait),
			"Ideal" => Ok(Self::Ideal),
			"Bond" => Ok(Self::Bond),
			"Flaw" => Ok(Self::Flaw),
			_ => Err(NotInList(s.into(), vec!["Trait", "Ideal", "Bond", "Flaw"])),
		}
	}
}
