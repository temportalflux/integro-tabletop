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

	pub fn description(&self) -> &'static str {
		match self {
			Self::Acrobatics => {
				"Your Dexterity (Acrobatics) check covers your attempt to stay on your feet in a tricky \
				situation, such as when you're trying to run across a sheet of ice, balance on a tightrope, \
				or stay upright on a rocking ship's deck. The DM might also call for a Dexterity (Acrobatics) \
				check to see if you can perform acrobatic stunts, including dives, rolls, somersaults, and flips."
			}
			Self::AnimalHandling => {
				"When there is any question whether you can calm down a domesticated animal, keep a mount \
				from getting spooked, or intuit an animal's intentions, the DM might call for a \
				Wisdom (Animal Handling) check. You also make a Wisdom (Animal Handling) check to \
				control your mount when you attempt a risky maneuver."
			}
			Self::Arcana => {
				"Your Intelligence (Arcana) check measures your ability to recall lore \
				about spells, magic items, eldritch symbols, magical traditions, \
				the planes of existence, and the inhabitants of those planes."
			}
			Self::Athletics => {
				"Your Strength (Athletics) check covers difficult situations you encounter \
				while climbing, jumping, or swimming. Examples include the following activities:

				- You attempt to climb a sheer or slippery cliff, avoid hazards while scaling a wall, \
				or cling to a surface while something is trying to knock you off.
				- You try to jump an unusually long distance or pull off a stunt midjump.
				- You struggle to swim or stay afloat in treacherous currents, storm-tossed waves, \
				or areas of thick seaweed. Or another creature tries to push or pull you \
				underwater or otherwise interfere with your swimming."
			}
			Self::Deception => {
				"Your Charisma (Deception) check determines whether you can convincingly hide the truth, \
				either verbally or through your actions. This deception can encompass everything from \
				misleading others through ambiguity to telling outright lies. Typical situations include \
				trying to fast-talk a guard, con a merchant, earn money through gambling, pass yourself \
				off in a disguise, dull someone's suspicions with false assurances, \
				or maintain a straight face while telling a blatant lie."
			}
			Self::History => {
				"Your Intelligence (History) check measures your ability to recall lore \
				about historical events, legendary people, ancient kingdoms, \
				past disputes, recent wars, and lost civilizations."
			}
			Self::Insight => {
				"Your Wisdom (Insight) check decides whether you can determine the true \
				intentions of a creature, such as when searching out a lie or predicting \
				someone's next move. Doing so involves gleaning clues from body language, \
				speech habits, and changes in mannerisms."
			}
			Self::Intimidation => {
				"When you attempt to influence someone through overt threats, hostile actions, \
				and physical violence, the DM might ask you to make a Charisma (Intimidation) check. \
				Examples include trying to pry information out of a prisoner, convincing street thugs \
				to back down from a confrontation, or using the edge of a broken bottle to convince a \
				sneering vizier to reconsider a decision."
			}
			Self::Investigation => {
				"When you look around for clues and make deductions based on those clues, you make an \
				Intelligence (Investigation) check. You might deduce the location of a hidden object, \
				discern from the appearance of a wound what kind of weapon dealt it, or determine the \
				weakest point in a tunnel that could cause it to collapse. Poring through ancient \
				scrolls in search of a hidden fragment of knowledge might also \
				call for an Intelligence (Investigation) check."
			}
			Self::Medicine => {
				"A Wisdom (Medicine) check lets you try to stabilize a dying companion or diagnose an illness."
			}
			Self::Nature => {
				"Your Intelligence (Nature) check measures your ability to recall lore \
				about terrain, plants and animals, the weather, and natural cycles."
			}
			Self::Perception => {
				"Your Wisdom (Perception) check lets you spot, hear, or otherwise \
				detect the presence of something. It measures your general awareness \
				of your surroundings and the keenness of your senses. For example, you might try \
				to hear a conversation through a closed door, eavesdrop under an open window, \
				or hear monsters moving stealthily in the forest. Or you might try to spot things \
				that are obscured or easy to miss, whether they are orcs lying in ambush on a road, \
				thugs hiding in the shadows of an alley, or candlelight under a closed secret door."
			}
			Self::Performance => {
				"Your Charisma (Performance) check determines how well you can delight an \
				audience with music, dance, acting, storytelling, or some other form of entertainment."
			}
			Self::Persuasion => {
				"When you attempt to influence someone or a group of people with tact, social graces, \
				or good nature, the DM might ask you to make a Charisma (Persuasion) check. \
				Typically, you use persuasion when acting in good faith, to foster friendships, \
				make cordial requests, or exhibit proper etiquette. Examples of persuading others \
				include convincing a chamberlain to let your party see the king, negotiating peace \
				between warring tribes, or inspiring a crowd of townsfolk."
			}
			Self::Religion => {
				"Your Intelligence (Religion) check measures your ability to recall lore about deities, \
				rites and prayers, religious hierarchies, holy symbols, and the practices of secret cults."
			}
			Self::SleightOfHand => {
				"Whenever you attempt an act of legerdemain or manual trickery, such as planting \
				something on someone else or concealing an object on your person, make a \
				Dexterity (Sleight of Hand) check. The DM might also call for a \
				Dexterity (Sleight of Hand) check to determine whether you can lift \
				a coin purse off another person or slip something out of another person's pocket."
			}
			Self::Stealth => {
				"Make a Dexterity (Stealth) check when you attempt to \
				conceal yourself from enemies, slink past guards, slip away without being noticed, \
				or sneak up on someone without being seen or heard."
			}
			Self::Survival => {
				"The DM might ask you to make a Wisdom (Survival) check to follow tracks, \
				hunt wild game, guide your group through frozen wastelands, identify signs \
				that owlbears live nearby, predict the weather, \
				or avoid quicksand and other natural hazards."
			}
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
