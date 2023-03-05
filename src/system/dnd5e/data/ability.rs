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

	pub fn short_description(&self) -> &'static str {
		match self {
			Self::Strength => {
				"Strength measures bodily power, athletic training, \
				and the extent to which you can exert raw physical force."
			}
			Self::Dexterity => "Dexterity measures agility, reflexes, and balance.",
			Self::Constitution => "Constitution measures health, stamina, and vital force.",
			Self::Intelligence => {
				"Intelligence measures mental acuity, \
				accuracy of recall, and the ability to reason."
			}
			Self::Wisdom => {
				"Wisdom reflects how attuned you are to the world around \
				you and represents perceptiveness and intuition."
			}
			Self::Charisma => {
				"Charisma measures your ability to interact effectively with others. \
				It includes such factors as confidence and eloquence, \
				and it can represent a charming or commanding personality."
			}
		}
	}

	pub fn checks_description(&self) -> &'static str {
		match self {
			Self::Strength => {
				"A Strength check can model any attempt to lift, push, pull, \
				or break something, to force your body through a space, \
				or to otherwise apply brute force to a situation. The Athletics skill \
				reflects aptitude in certain kinds of Strength checks."
			}
			Self::Dexterity => {
				"A Dexterity check can model any attempt to move nimbly, quickly, \
				or quietly, or to keep from falling on tricky footing. The Acrobatics, \
				Sleight of Hand, and Stealth skills reflect aptitude in \
				certain kinds of Dexterity checks."
			}
			Self::Constitution => {
				"Constitution checks are uncommon, and no skills apply to Constitution checks, \
				because the endurance this ability represents is largely passive rather than \
				involving a specific effort on the part of a character or monster. \
				A Constitution check can model your attempt to push beyond normal limits, however.

				The DM might call for a Constitution check when you try to accomplish tasks like the following:
				- Hold your breath
				- March or labor for hours without rest
				- Go without sleep
				- Survive without food or water
				- Quaff an entire stein of ale in one go"
			}
			Self::Intelligence => {
				"An Intelligence check comes into play when you need to draw on logic, \
				education, memory, or deductive reasoning. The Arcana, History, Investigation, \
				Nature, and Religion skills reflect aptitude in certain kinds of Intelligence checks."
			}
			Self::Wisdom => {
				"A Wisdom check might reflect an effort to read body language, understand someone's \
				feelings, notice things about the environment, or care for an injured person. \
				The Animal Handling, Insight, Medicine, Perception, and Survival skills \
				reflect aptitude in certain kinds of Wisdom checks."
			}
			Self::Charisma => {
				"A Charisma check might arise when you try to influence or entertain others, \
				when you try to make an impression or tell a convincing lie, or when you are \
				navigating a tricky social situation. The Deception, Intimidation, Performance, \
				and Persuasion skills reflect aptitude in certain kinds of Charisma checks."
			}
		}
	}

	pub fn addendum_description(&self) -> Vec<(&'static str, &'static str)> {
		match self {
			Self::Strength => vec![
				(
					"Other Strength Checks",
					"The DM might also call for a Strength \
					check when you try to accomplish tasks like the following:
					- Force open a stuck, locked, or barred door
					- Break free of bonds
					- Push through a tunnel that is too small
					- Hang on to a wagon while being dragged behind it
					- Tip over a statue
					- Keep a boulder from rolling"
				),
				(
					"Attack Rolls and Damage",
					"You add your Strength modifier to your attack roll and \
					your damage roll when attacking with a melee weapon such \
					as a mace, a battleaxe, or a javelin. You use melee weapons \
					to make melee attacks in hand-to-hand combat, and some of them \
					can be thrown to make a ranged attack."
				),
				(
					"Lifting and Carrying",
					"Your Strength score determines the amount of weight you can bear. \
					The following terms define what you can lift or carry.
					
					Carrying Capacity. Your carrying capacity is your Strength score multiplied by 15. \
					This is the weight (in pounds) that you can carry, which is high enough that most \
					characters don't usually have to worry about it.
					
					Push, Drag, or Lift. You can push, drag, or lift a weight in pounds up to \
					twice your carrying capacity (or 30 times your Strength score). \
					While pushing or dragging weight in excess of your carrying capacity, your speed drops to 5 feet.
					
					Size and Strength. Larger creatures can bear more weight, whereas Tiny creatures can carry less. \
					For each size category above Medium, double the creature's carrying capacity and the \
					amount it can push, drag, or lift. For a Tiny creature, halve these weights.
					
					Variant: Encumbrance
					The rules for lifting and carrying are intentionally simple. Here is a variant if you are looking for \
					more detailed rules for determining how a character is hindered by the weight of equipment. \
					When you use this variant, ignore the Strength column of the Armor table in chapter 5.
					
					If you carry weight in excess of 5 times your Strength score, you are encumbered, \
					which means your speed drops by 10 feet.
					
					If you carry weight in excess of 10 times your Strength score, up to your maximum carrying capacity, \
					you are instead heavily encumbered, which means your speed drops by 20 feet and you have disadvantage \
					on ability checks, attack rolls, and saving throws that use Strength, Dexterity, or Constitution."
				),
			],
			Self::Dexterity => vec![
				(
					"Other Dexterity Checks",
					"The DM might call for a Dexterity check when you try to accomplish tasks like the following:
					- Control a heavily laden cart on a steep descent
					- Steer a chariot around a tight turn
					- Pick a lock
					- Disable a trap
					- Securely tie up a prisoner
					- Wriggle free of bonds
					- Play a stringed instrument
					- Craft a small or detailed object"
				),
				(
					"Attack Rolls and Damage",
					"You add your Dexterity modifier to your attack roll and your damage roll when attacking with \
					a ranged weapon, such as a sling or a longbow. You can also add your Dexterity modifier to your \
					attack roll and your damage roll when attacking with a melee weapon that has the finesse \
					property, such as a dagger or a rapier."
				),
				(
					"Armor Class",
					"Depending on the armor you wear, you might add some or all \
					of your Dexterity modifier to your Armor Class."
				),
				(
					"Initiative",
					"At the beginning of every combat, you roll initiative by making a Dexterity check. \
					Initiative determines the order of creatures' turns in combat."
				),
			],
			Self::Constitution => vec![
				(
					"Hit Points",
					"Your Constitution modifier contributes to your hit points. \
					Typically, you add your Constitution modifier to each Hit Die you roll for your hit points.
					
					If your Constitution modifier changes, your hit point maximum changes as well, as though \
					you had the new modifier from 1st level. For example, if you raise your Constitution score \
					when you reach 4th level and your Constitution modifier increases from +1 to +2, you adjust \
					your hit point maximum as though the modifier had always been +2. So you add 3 hit points \
					for your first three levels, and then roll your hit points for 4th level using your new \
					modifier. Or if you're 7th level and some effect lowers your Constitution score so as to \
					reduce your Constitution modifier by 1, your hit point maximum is reduced by 7."
				),
			],
			Self::Intelligence => vec![
				(
					"Other Intelligence Checks",
					"The DM might call for an Intelligence check when you try to accomplish tasks like the following:
					- Communicate with a creature without using words
					- Estimate the value of a precious item
					- Pull together a disguise to pass as a city guard
					- Forge a document
					- Recall lore about a craft or trade
					- Win a game of skill"
				),
				(
					"Spellcasting Ability",
					"Wizards use Intelligence as their spellcasting ability, \
					which helps determine the saving throw DCs of spells they cast."
				),
			],
			Self::Wisdom => vec![
				(
					"Other Wisdom Checks",
					"The DM might call for a Wisdom check when you try to accomplish tasks like the following:
					- Get a gut feeling about what course of action to follow
					- Discern whether a seemingly dead or living creature is undead"
				),
				(
					"Spellcasting Ability",
					"Clerics, druids, and rangers use Wisdom as their spellcasting ability, \
					which helps determine the saving throw DCs of spells they cast."
				),
			],
			Self::Charisma => vec![
				(
					"Other Charisma Checks",
					"The DM might call for a Charisma check when you try to accomplish tasks like the following:
					- Find the best person to talk to for news, rumors, and gossip
					- Blend into a crowd to get the sense of key topics of conversation"
				),
				(
					"Spellcasting Ability",
					"Bards, paladins, sorcerers, and warlocks use Charisma as their spellcasting ability, \
					which helps determine the saving throw DCs of spells they cast."
				),
			],
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Score(pub u32);
impl Default for Score {
	fn default() -> Self {
		Self(10)
	}
}
impl std::ops::Deref for Score {
	type Target = u32;

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
		let value = self.0 as i32;
		let value = (value - 10) as f32;
		let value = (value / 2f32).floor();
		value as i32
	}
}
