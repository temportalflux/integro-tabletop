use super::Selector;
use crate::system::dnd5e::{character::Character, proficiency, roll, Skill};

#[derive(Clone)]
pub struct AddSkill {
	pub skill: Selector<Skill>,
	pub proficiency: proficiency::Level,
}

impl super::Mutator for AddSkill {
	fn node_id(&self) -> &'static str {
		"add_skill"
	}

	fn id(&self) -> Option<&str> {
		self.skill.id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		if let Some(skill) = stats.resolve_selector(&self.skill) {
			let source = stats.source_path();
			stats
				.skills_mut()
				.add_proficiency(skill, self.proficiency, source);
		}
	}
}

#[derive(Clone)]
pub struct AddSkillModifier {
	pub skill: Skill,
	pub modifier: roll::Modifier,
	pub criteria: Option<String>,
}
impl super::Mutator for AddSkillModifier {
	fn node_id(&self) -> &'static str {
		"add_skill_modifier"
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats
			.skills_mut()
			.add_modifier(self.skill, self.modifier, self.criteria.clone(), source);
	}
}
