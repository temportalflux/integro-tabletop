use std::str::FromStr;
use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{
		data::{character::Character, proficiency, roll, Skill},
		DnD5e, FromKDL, KDLNode,
	},
	utility::{Mutator, Selector},
	GeneralError,
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddSkillModifier {
	pub skill: Skill,
	pub modifier: roll::Modifier,
	pub criteria: Option<String>,
}

impl crate::utility::TraitEq for AddSkillModifier {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
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

impl FromKDL<DnD5e> for AddSkillModifier {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let skill = Skill::from_str(node.get_str(value_idx.next())?)?;
		let modifier = roll::Modifier::from_str(node.get_str(value_idx.next())?)?;
		let criteria = node.get_str_opt(value_idx.next())?.map(str::to_owned);
		Ok(Self {
			skill,
			modifier,
			criteria,
		})
	}
}

// TODO: Tests for AddSkillModifier
