use crate::{
	kdl_ext::{NodeExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{character::Character, roll, Skill},
			FromKDL,
		},
	},
	utility::Mutator,
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct AddSkillModifier {
	pub skill: Skill,
	pub modifier: roll::Modifier,
	pub criteria: Option<String>,
}

crate::impl_trait_eq!(AddSkillModifier);
crate::impl_kdl_node!(AddSkillModifier, "add_skill_modifier");

impl Mutator for AddSkillModifier {
	type Target = Character;

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		stats.skills_mut().add_modifier(
			self.skill,
			self.modifier,
			self.criteria.clone(),
			parent.to_owned(),
		);
	}
}

impl FromKDL for AddSkillModifier {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let skill = Skill::from_str(node.get_str_req(value_idx.next())?)?;
		let modifier = roll::Modifier::from_str(node.get_str_req(value_idx.next())?)?;
		let criteria = node.get_str_opt(value_idx.next())?.map(str::to_owned);
		Ok(Self {
			skill,
			modifier,
			criteria,
		})
	}
}

// TODO: Tests for AddSkillModifier
