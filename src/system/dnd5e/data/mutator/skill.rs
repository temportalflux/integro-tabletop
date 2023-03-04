use crate::{
	system::dnd5e::{
		data::{character::Character, proficiency, roll, Skill},
		KDLNode,
	},
	utility::{Mutator, Selector},
};

#[derive(Clone, Debug)]
pub struct AddSkill {
	pub skill: Selector<Skill>,
	pub proficiency: proficiency::Level,
}

impl KDLNode for AddSkill {
	fn id() -> &'static str {
		"add_skill"
	}
}

impl Mutator for AddSkill {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn data_id(&self) -> Option<&str> {
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

#[derive(Clone, Debug)]
pub struct AddSkillModifier {
	pub skill: Skill,
	pub modifier: roll::Modifier,
	pub criteria: Option<String>,
}

impl KDLNode for AddSkillModifier {
	fn id() -> &'static str {
		"add_skill_modifier"
	}
}

impl Mutator for AddSkillModifier {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats
			.skills_mut()
			.add_modifier(self.skill, self.modifier, self.criteria.clone(), source);
	}
}
