use super::{
	action::{Action, LimitedUses},
	character::Character,
	description,
};
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::{BoxedCriteria, BoxedMutator},
	utility::MutatorGroup,
};
use derivative::Derivative;
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, RwLock},
};

#[derive(Default, Clone, Debug, Derivative)]
#[derivative(PartialEq)]
pub struct Feature {
	pub name: String,
	pub description: description::Info,

	/// If true, the feature should not be shown in full in the feature overviews.
	/// Instead, display only the name in a brief section,
	/// and clicking the name opens the modal for the feature.
	/// If a feature is marked as collapsed, but another feature
	/// marks it as its parent, the collapsed property is ignored.
	pub collapsed: bool,
	/// The path of the parent feature, for grouping features together in the UI.
	pub parent: Option<PathBuf>,

	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,
	pub action: Option<Action>,

	#[derivative(PartialEq = "ignore")]
	pub absolute_path: Arc<RwLock<PathBuf>>,
	#[derivative(PartialEq = "ignore")]
	pub missing_selection_text: Option<(String, HashMap<String, String>)>,
}

impl Feature {
	pub fn get_display_path(&self) -> PathBuf {
		self.absolute_path.read().unwrap().clone()
	}

	pub fn get_missing_selection_text_for(&self, key: &str) -> Option<&String> {
		let Some((default_text, specialized)) = &self.missing_selection_text else { return None; };
		if let Some(key_specific) = specialized.get(key) {
			return Some(key_specific);
		}
		Some(default_text)
	}
}

impl MutatorGroup for Feature {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(&self.name);

		if let Some(action) = &self.action {
			if let Some(uses) = &action.limited_uses {
				uses.set_data_path(parent);
			}
		}

		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		if let Some(criteria) = &self.criteria {
			// TODO: Somehow save the error text for display in feature UI
			if stats.evaluate(criteria).is_err() {
				return;
			}
		}
		if let Some(action) = &self.action {
			if let Some(uses) = &action.limited_uses {
				if let LimitedUses::Usage(data) = uses {
					stats.features_mut().register_usage(data, &path_to_self);
				}
			}
		}
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
		*self.absolute_path.write().unwrap() = path_to_self;
	}
}

impl FromKDL for Feature {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = match node.query_opt("scope() > description")? {
			None => description::Info::default(),
			Some(node) => description::Info::from_kdl(node, &mut ctx.next_node())?,
		};

		let collapsed = node.get_bool_opt("collapsed")?.unwrap_or_default();
		let parent = node.get_str_opt("parent")?.map(PathBuf::from);

		// Specifies if this feature can appear twice.
		// If true, any other features with the same name are ignored/discarded.
		// TODO: Unimplemented
		let _is_unique = node.get_bool_opt("unique")?.unwrap_or_default();

		let criteria = match node.query("scope() > criteria")? {
			None => None,
			Some(entry_node) => {
				Some(ctx.parse_evaluator::<Character, Result<(), String>>(entry_node)?)
			}
		};

		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let action = match node.query_opt("scope() > action")? {
			None => None,
			Some(node) => Some(Action::from_kdl(node, &mut ctx.next_node())?),
		};

		Ok(Self {
			name,
			description,
			collapsed,
			parent,
			mutators,
			criteria,
			action,
			// Generated data
			absolute_path: Arc::new(RwLock::new(PathBuf::new())),
			missing_selection_text: None,
		})
	}
}
// TODO AsKdl: from/as tests for Bundle
impl AsKdl for Feature {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("name", self.name.clone()));
		if self.description != description::Info::default() {
			node.push_child_t("description", &self.description);
		}

		if self.collapsed {
			node.push_entry(("collapsed", true));
		}
		if let Some(parent) = &self.parent {
			node.push_entry(("parent", parent.display().to_string()));
		}

		if let Some(criteria) = &self.criteria {
			// TODO AsKdl: evaluator; node.push_child_t("criteria", criteria);
		}
		for mutator in &self.mutators {
			// TODO AsKdl: mutators; node.push_child_t("mutator", mutator);
		}

		if let Some(action) = &self.action {
			node.push_child_t("action", action);
		}

		node
	}
}
