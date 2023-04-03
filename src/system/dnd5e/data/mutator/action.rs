use crate::{
	kdl_ext::FromKDL,
	system::dnd5e::data::{action::{Action, ActionSource}, character::Character},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddAction(pub Action);

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
		let sections = sections.into_iter().map(|section| {
			match section.title {
				Some(title) => format!("{title}. {}", section.content),
				None => section.content,
			}
		}).collect::<Vec<_>>();
		Some(sections.join("\n"))
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.0.set_data_path(parent);
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		let mut action = self.0.clone();
		action.source = Some(ActionSource::Feature(parent.to_owned()));
		stats.actions_mut().push(action);
	}
}

impl FromKDL for AddAction {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(Action::from_kdl(node, ctx)?))
	}
}
