use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::{action::Action, character::Character, description, Feature},
	utility::{Mutator, MutatorGroup},
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddAction(pub Feature);

crate::impl_trait_eq!(AddAction);
crate::impl_kdl_node!(AddAction, "add_action");

impl Mutator for AddAction {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some(self.0.name.clone())
	}

	fn description(&self) -> Option<String> {
		let sections = self.0.description.long().collect::<Vec<_>>();
		if sections.is_empty() {
			return None;
		}
		let sections = sections
			.into_iter()
			.map(|section| match section.title {
				Some(title) => format!("{title}. {}", section.content),
				None => section.content,
			})
			.collect::<Vec<_>>();
		Some(sections.join("\n"))
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.0.set_data_path(parent);
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		stats.add_feature(&self.0, parent);
	}
}

impl FromKDL for AddAction {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = description::Info::from_kdl_all(node, ctx)?;
		let action = Action::from_kdl(node, ctx)?;
		Ok(Self(Feature {
			name,
			description,
			action: Some(action),
			..Default::default()
		}))
	}
}
