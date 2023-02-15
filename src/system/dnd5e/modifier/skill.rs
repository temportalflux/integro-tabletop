use super::Selector;
use crate::system::dnd5e::{character::DerivedBuilder, proficiency, roll, Skill};
use std::str::FromStr;

#[derive(Clone)]
pub struct AddSkill {
	pub skill: Selector<Skill>,
	pub proficiency: proficiency::Level,
}

impl super::Modifier for AddSkill {
	fn scope_id(&self) -> Option<&str> {
		self.skill.id()
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		let skill = match &self.skill {
			Selector::Specific(skill) => Some(*skill),
			_ => match stats.get_selection() {
				Some(value) => Skill::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(skill) = skill {
			stats.add_skill(skill, self.proficiency);
		}
	}
}

#[derive(Clone)]
pub struct AddSkillModifier {
	pub skill: Skill,
	pub modifier: roll::Modifier,
	pub criteria: Option<String>,
}
impl super::Modifier for AddSkillModifier {
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		// TODO: Apply skill modifier (adv/dis)
	}
}
