use super::Selector;
use crate::system::dnd5e::{character::CompiledStats, Character, ProficiencyLevel, Skill};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone)]
pub struct AddSkill {
	pub skill: Selector<Skill>,
	pub proficiency: ProficiencyLevel,
}

impl super::Modifier for AddSkill {
	fn scope_id(&self) -> Option<&str> {
		self.skill.id()
	}

	fn apply(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		let skill = match &self.skill {
			Selector::Specific(skill) => Some(*skill),
			_ => match char.get_selection(stats, &scope) {
				Some(value) => Skill::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(skill) = skill {
			stats.skills[skill] = stats.skills[skill].max(self.proficiency);
		}
	}
}
