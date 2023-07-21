use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	system::dnd5e::data::Size,
	utility::NotInList,
};
use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use itertools::Itertools;
use std::{collections::HashSet, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Description {
	pub name: String,
	pub pronouns: HashSet<String>,
	pub custom_pronouns: String,
	pub height: u32,
	pub weight: u32,
	pub age: u32,
	pub personality: EnumMap<PersonalityKind, Vec<String>>,
	pub appearance: String,
}

impl Description {
	pub fn size(&self) -> Size {
		match self.height {
			v if v >= 45 => Size::Medium,
			_ => Size::Small,
		}
	}

	pub fn iter_pronouns(&self) -> impl Iterator<Item = &String> + '_ {
		self.pronouns
			.iter()
			.sorted()
			.chain(match self.custom_pronouns.is_empty() {
				true => vec![],
				false => vec![&self.custom_pronouns],
			})
	}
}

impl FromKDL for Description {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let name = node.query_str_req("scope() > name", 0)?.to_owned();

		let mut pronouns = HashSet::new();
		let mut custom_pronouns = String::new();
		for value in node.query_str_all("scope() > pronoun", 0)? {
			match value {
				"she/her" | "he/him" | "they/them" => {
					pronouns.insert(value.to_owned());
				}
				_ => {
					if !custom_pronouns.is_empty() {
						custom_pronouns.push(',');
					}
					custom_pronouns.push_str(value);
				}
			}
		}

		let age = node
			.query_i64_opt("scope() > age", 0)?
			.map(|v| v as u32)
			.unwrap_or_default();
		let height = node
			.query_i64_opt("scope() > height", 0)?
			.map(|v| v as u32)
			.unwrap_or_default();
		let weight = node
			.query_i64_opt("scope() > weight", 0)?
			.map(|v| v as u32)
			.unwrap_or_default();

		let mut personality = EnumMap::<PersonalityKind, Vec<String>>::default();
		if let Some(node) = node.query_opt("scope() > personality")? {
			for (kind, values) in personality.iter_mut() {
				for node in node.query_str_all(format!("scope() > {}", kind.node_id()), 0)? {
					values.push(node.to_owned());
				}
			}
		}
		let appearance = node
			.query_str_opt("scope() > appearance", 0)?
			.map(str::to_owned)
			.unwrap_or_default();

		Ok(Self {
			name,
			pronouns,
			custom_pronouns,
			height,
			weight,
			age,
			personality,
			appearance,
		})
	}
}

impl AsKdl for Description {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_child_t("name", &self.name);
		for pronoun in self.pronouns.iter().sorted() {
			node.push_child_t("pronoun", pronoun);
		}
		if !self.custom_pronouns.is_empty() {
			node.push_child_t("pronoun", &self.custom_pronouns);
		}
		if self.age != 0 {
			node.push_child_t("age", &self.age);
		}
		if self.height != 0 {
			node.push_child_t("height", &self.height);
		}
		if self.weight != 0 {
			node.push_child_t("weight", &self.weight);
		}
		node.push_child_opt({
			let mut node = NodeBuilder::default();
			for (kind, items) in &self.personality {
				for item in items {
					node.push_child_t(kind.node_id(), item);
				}
			}
			node.build("personality")
		});
		if !self.appearance.is_empty() {
			node.push_child_t("appearance", &self.appearance);
		}
		node
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
	fn node_id(&self) -> &'static str {
		match self {
			Self::Trait => "trait",
			Self::Ideal => "ideal",
			Self::Bond => "bond",
			Self::Flaw => "flaw",
		}
	}

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

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "description";

		#[test]
		fn name() -> anyhow::Result<()> {
			let doc = "
				|description {
				|    name \"Alakazam\"
				|}
			";
			let data = Description {
				name: "Alakazam".into(),
				..Default::default()
			};
			assert_eq_fromkdl!(Description, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn height_weight() -> anyhow::Result<()> {
			let doc = "
				|description {
				|    name \"Alakazam\"
				|    height 60
				|    weight 90
				|}
			";
			let data = Description {
				name: "Alakazam".into(),
				height: 60,
				weight: 90,
				..Default::default()
			};
			assert_eq_fromkdl!(Description, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn pronouns() -> anyhow::Result<()> {
			let doc = "
				|description {
				|    name \"Alakazam\"
				|    pronoun \"he/him\"
				|    pronoun \"she/her\"
				|    pronoun \"they/them\"
				|    pronoun \"xi/xir\"
				|}
			";
			let data = Description {
				name: "Alakazam".into(),
				pronouns: ["he/him".into(), "she/her".into(), "they/them".into()].into(),
				custom_pronouns: "xi/xir".into(),
				..Default::default()
			};
			assert_eq_fromkdl!(Description, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn personality() -> anyhow::Result<()> {
			let doc = "
				|description {
				|    name \"Alakazam\"
				|    personality {
				|        trait \"Trait 1\"
				|        trait \"Trait 2\"
				|        ideal \"Ideal A\"
				|        bond \"Bond B\"
				|        flaw \"Flaw C\"
				|    }
				|}
			";
			let data = Description {
				name: "Alakazam".into(),
				personality: enum_map::enum_map! {
					PersonalityKind::Trait => vec!["Trait 1".into(), "Trait 2".into()],
					PersonalityKind::Ideal => vec!["Ideal A".into()],
					PersonalityKind::Bond => vec!["Bond B".into()],
					PersonalityKind::Flaw => vec!["Flaw C".into()],
				},
				..Default::default()
			};
			assert_eq_fromkdl!(Description, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
