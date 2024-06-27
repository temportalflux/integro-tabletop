use crate::system::dnd5e::data::{
	proficiency,
	roll::{ModifierList, NumbericalBonusList},
	Ability, Skill,
};
use enum_map::EnumMap;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Skills {
	abilities: EnumMap<Ability, AbilitySkillEntry>,
	skills: EnumMap<Skill, AbilitySkillEntry>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AbilitySkillEntry(proficiency::List, ModifierList, NumbericalBonusList);

impl std::ops::Index<Ability> for Skills {
	type Output = AbilitySkillEntry;
	fn index(&self, index: Ability) -> &Self::Output {
		&self.abilities[index]
	}
}

impl std::ops::IndexMut<Ability> for Skills {
	fn index_mut(&mut self, index: Ability) -> &mut Self::Output {
		&mut self.abilities[index]
	}
}

impl std::ops::Index<Skill> for Skills {
	type Output = AbilitySkillEntry;
	fn index(&self, index: Skill) -> &Self::Output {
		&self.skills[index]
	}
}

impl std::ops::IndexMut<Skill> for Skills {
	fn index_mut(&mut self, index: Skill) -> &mut Self::Output {
		&mut self.skills[index]
	}
}

impl AbilitySkillEntry {
	pub fn proficiencies_mut(&mut self) -> &mut proficiency::List {
		&mut self.0
	}

	pub fn modifiers_mut(&mut self) -> &mut ModifierList {
		&mut self.1
	}

	pub fn bonuses_mut(&mut self) -> &mut NumbericalBonusList {
		&mut self.2
	}

	pub fn proficiencies(&self) -> &proficiency::List {
		&self.0
	}

	pub fn modifiers(&self) -> &ModifierList {
		&self.1
	}

	pub fn bonuses(&self) -> &NumbericalBonusList {
		&self.2
	}
}

impl Skills {
	pub fn iter_ability_mut(
		&mut self, ability: Option<Ability>,
	) -> impl Iterator<Item = (Ability, &mut AbilitySkillEntry)> {
		let iter = self.abilities.iter_mut();
		let iter = iter.filter(move |(key, _)| ability.is_none() || Some(*key) == ability);
		iter
	}
}
