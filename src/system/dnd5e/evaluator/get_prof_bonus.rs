use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	system::dnd5e::data::character::Character,
	utility::Evaluator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct GetProficiencyBonus;

crate::impl_trait_eq!(GetProficiencyBonus);
impl Evaluator for GetProficiencyBonus {
	type Context = Character;
	type Item = i32;

	fn description(&self) -> Option<String> {
		Some(format!("your proficiency bonus"))
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		state.proficiency_bonus()
	}
}

crate::impl_kdl_node!(GetProficiencyBonus, "get_proficiency_bonus");

impl FromKDL for GetProficiencyBonus {
	fn from_kdl(
		_node: &kdl::KdlNode,
		_ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self)
	}
}
impl AsKdl for GetProficiencyBonus {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default()
	}
}
