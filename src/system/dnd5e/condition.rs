use dyn_clone::{clone_trait_object, DynClone};

use super::{criteria::BoxedCriteria, mutator::BoxedMutator};

pub trait Condition: DynClone {
	fn description(&self) -> String
	where
		Self: Sized;
}
clone_trait_object!(Condition);

#[derive(Clone)]
pub struct BoxedCondition(std::rc::Rc<dyn Condition + 'static>);
impl PartialEq for BoxedCondition {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for BoxedCondition {
	type Target = std::rc::Rc<dyn Condition + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T> From<T> for BoxedCondition
where
	T: Condition + 'static,
{
	fn from(value: T) -> Self {
		Self(std::rc::Rc::new(value))
	}
}

#[derive(Clone, PartialEq)]
pub struct Blinded;
impl Condition for Blinded {
	fn description(&self) -> String {
		"A blinded creature can't see and automatically fails any ability check that requires sight. 
		Attack rolls against the creature have advantage, and the creature's attack rolls have disadvantage.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Charmed;
impl Condition for Charmed {
	fn description(&self) -> String {
		"A charmed creature can't attack the charmer or target the charmer with harmful abilities or magical effects. 
		The charmer has advantage on any ability check to interact socially with the creature.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Deafened;
impl Condition for Deafened {
	fn description(&self) -> String {
		"A deafened creature can't hear and automatically fails any ability check that requires hearing.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Frightened;
impl Condition for Frightened {
	fn description(&self) -> String {
		"A frightened creature has disadvantage on ability checks and attack rolls while the source of its fear is within line of sight. 
		The creature can't willingly move closer to the source of its fear.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Grappled;
impl Condition for Grappled {
	fn description(&self) -> String {
		"A grappled creature's speed becomes 0, and it can't benefit from any bonus to its speed. 
		The condition ends if the grappler is incapacitated (see the condition). 
		The condition also ends if an effect removes the grappled creature from the reach of \
		the grappler or grappling effect, such as when a creature is hurled away by the thunder-wave spell."
			.into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Incapacitated;
impl Condition for Incapacitated {
	fn description(&self) -> String {
		"An incapacitated creature can't take actions or reactions.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Invisible;
impl Condition for Invisible {
	fn description(&self) -> String {
		"An invisible creature is impossible to see without the aid of magic or a special sense. \
		For the purpose of hiding, the creature is heavily obscured. The creature's location can be \
		detected by any noise it makes or any tracks it leaves. 
		Attack rolls against the creature have disadvantage, and the creature's attack rolls have advantage.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Paralyzed;
impl Condition for Paralyzed {
	fn description(&self) -> String {
		"A paralyzed creature is incapacitated (see the condition) and can't move or speak. 
		The creature automatically fails Strength and Dexterity saving throws. Attack rolls against the creature have advantage. 
		Any attack that hits the creature is a critical hit if the attacker is within 5 feet of the creature.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Petrified;
impl Condition for Petrified {
	fn description(&self) -> String {
		"A petrified creature is transformed, along with any nonmagical object it is wearing or carrying, \
		into a solid inanimate substance (usually stone). Its weight increases by a factor of ten, and it ceases aging. 
		The creature is incapacitated (see the condition), can't move or speak, and is unaware of its surroundings. 
		Attack rolls against the creature have advantage. 
		The creature automatically fails Strength and Dexterity saving throws. 
		The creature has resistance to all damage. 
		The creature is immune to poison and disease, although a poison or disease already \
		in its system is suspended, not neutralized.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Poisoned;
impl Condition for Poisoned {
	fn description(&self) -> String {
		"A poisoned creature has disadvantage on attack rolls and ability checks.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Prone;
impl Condition for Prone {
	fn description(&self) -> String {
		"A prone creature's only movement option is to crawl, unless it stands up and thereby ends the condition. 
		The creature has disadvantage on attack rolls. 
		An attack roll against the creature has advantage if the attacker is within 5 feet of the creature. \
		Otherwise, the attack roll has disadvantage.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Restrained;
impl Condition for Restrained {
	fn description(&self) -> String {
		"A restrained creature's speed becomes 0, and it can't benefit from any bonus to its speed. 
		Attack rolls against the creature have advantage, and the creature's attack rolls have disadvantage. 
		The creature has disadvantage on Dexterity saving throws.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Stunned;
impl Condition for Stunned {
	fn description(&self) -> String {
		"A stunned creature is incapacitated (see the condition), can't move, and can speak only falteringly. 
		The creature automatically fails Strength and Dexterity saving throws. 
		Attack rolls against the creature have advantage.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Unconscious;
impl Condition for Unconscious {
	fn description(&self) -> String {
		"An unconscious creature is incapacitated (see the condition), can't move or speak, and is unaware of its surroundings. 
		The creature drops whatever it's holding and falls prone. 
		The creature automatically fails Strength and Dexterity saving throws. 
		Attack rolls against the creature have advantage. 
		Any attack that hits the creature is a critical hit if the attacker is within 5 feet of the creature.".into()
	}
}

#[derive(Clone, PartialEq)]
pub struct Exhasted(Exhaustion);
impl Condition for Exhasted {
	fn description(&self) -> String {
		let mut desc = "Some special abilities and environmental hazards, such as starvation and the long-term effects of \
		freezing or scorching temperatures, can lead to a special condition called exhaustion. \
		Exhaustion is measured in six levels. An effect can give a creature one or more levels of exhaustion, \
		as specified in the effect's description. 
		If an already exhausted creature suffers another effect that causes exhaustion, \
		its current level of exhaustion increases by the amount specified in the effect's description. 
		A creature suffers the effect of its current level of exhaustion as well as all lower levels. \
		For example, a creature suffering level 2 exhaustion has its speed halved and has disadvantage on ability checks. 
		An effect that removes exhaustion reduces its level as specified in the effect's description, \
		with all exhaustion effects ending if a creature's exhaustion level is reduced below 1. 
		Finishing a long rest reduces a creature's exhaustion level by 1, \
		provided that the creature has also ingested some food and drink.\n".to_owned();

		desc += self
			.0
			.to_vec()
			.into_iter()
			.fold("Effects:".to_owned(), |desc, level| {
				desc + "\n" + level.name() + ": " + level.description()
			})
			.as_str();

		desc
	}
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Exhaustion {
	Level1,
	Level2,
	Level3,
	Level4,
	Level5,
	Level6,
}
impl Exhaustion {
	pub fn name(&self) -> &'static str {
		match self {
			Self::Level1 => "Level 1",
			Self::Level2 => "Level 2",
			Self::Level3 => "Level 3",
			Self::Level4 => "Level 4",
			Self::Level5 => "Level 5",
			Self::Level6 => "Level 6",
		}
	}

	pub fn description(&self) -> &'static str {
		match self {
			Self::Level1 => "Disadvantage on ability checks",
			Self::Level2 => "Speed halved",
			Self::Level3 => "Disadvantage on attack rolls and saving throws",
			Self::Level4 => "Hit point maximum halved",
			Self::Level5 => "Speed reduced to 0",
			Self::Level6 => "Death",
		}
	}

	pub fn next(&self) -> Option<Self> {
		match self {
			Self::Level1 => Some(Self::Level2),
			Self::Level2 => Some(Self::Level3),
			Self::Level3 => Some(Self::Level4),
			Self::Level4 => Some(Self::Level5),
			Self::Level5 => Some(Self::Level6),
			Self::Level6 => None,
		}
	}

	pub fn prev(&self) -> Option<Self> {
		match self {
			Self::Level1 => None,
			Self::Level2 => Some(Self::Level1),
			Self::Level3 => Some(Self::Level2),
			Self::Level4 => Some(Self::Level3),
			Self::Level5 => Some(Self::Level4),
			Self::Level6 => Some(Self::Level5),
		}
	}

	/// Returns the list of this and all previous levels.
	pub fn to_vec(self) -> Vec<Self> {
		let mut l = self;
		let mut v = vec![];
		while let Some(level) = l.prev() {
			v.push(level);
			l = level;
		}
		v
	}
}

#[derive(Clone, PartialEq)]
pub struct Custom {
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,
}
impl Condition for Custom {
	fn description(&self) -> String {
		self.description.clone()
	}
}
