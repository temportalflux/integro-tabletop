use super::Selector;
use crate::system::dnd5e::{character::StatsBuilder, ProficiencyLevel, Skill};
use std::str::FromStr;

#[derive(Clone)]
pub struct AddSkill {
	pub skill: Selector<Skill>,
	pub proficiency: ProficiencyLevel,
}

impl super::Modifier for AddSkill {
	fn scope_id(&self) -> Option<&str> {
		self.skill.id()
	}

	fn apply<'c>(&self, stats: &mut StatsBuilder<'c>) {
		let skill = match &self.skill {
			Selector::Specific(skill) => Some(*skill),
			_ => match stats.get_selection() {
				Some(value) => Skill::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(skill) = skill {
			stats.skills[skill] = stats.skills[skill].max(self.proficiency);
		}
	}
}
