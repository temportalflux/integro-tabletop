use crate::kdl_ext::NodeContext;
use crate::{system::dnd5e::data::character::Character, system::Evaluator};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

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

kdlize::impl_kdl_node!(GetProficiencyBonus, "get_proficiency_bonus");

impl FromKdl<NodeContext> for GetProficiencyBonus {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(_node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(Self)
	}
}
impl AsKdl for GetProficiencyBonus {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default()
	}
}
