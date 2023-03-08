use super::{
	action::{Action, ActionSource},
	character::Character,
};
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::dnd5e::{BoxedCriteria, BoxedMutator, DnD5e, FromKDL},
	utility::MutatorGroup,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Feature {
	pub name: String,
	pub description: String,

	pub actions: Vec<Action>,

	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,

	pub missing_selection_text: Option<(String, HashMap<String, String>)>,
}

impl Feature {
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

	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		if let Some(criteria) = &self.criteria {
			// TODO: Somehow save the error text for display in feature UI
			if stats.evaluate(criteria).is_err() {
				return;
			}
		}
		for mutator in &self.mutators {
			stats.apply(mutator);
		}
		for action in &self.actions {
			let mut action = action.clone();
			action.source = Some(ActionSource::Feature(stats.source_path()));
			stats.actions_mut().push(action);
		}
	}
}

impl FromKDL<DnD5e> for Feature {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.unwrap_or_default()
			.to_owned();
		// Specifies if this feature can appear twice.
		// If true, any other features with the same name are ignored/discarded.
		// TODO: Unimplemented
		let _is_unique = node.get_bool_opt("unique")?.unwrap_or_default();

		let criteria = match node.query("criteria")? {
			None => None,
			Some(entry_node) => {
				let mut value_idx = ValueIdx::default();
				let id = entry_node.get_str(value_idx.next())?;
				let factory = system.get_evaluator_factory(id)?;
				Some(factory.from_kdl::<Result<(), String>>(entry_node, &mut value_idx, system)?)
			}
		};

		let mut actions = Vec::new();
		for entry_node in node.query_all("action")? {
			let mut value_idx = ValueIdx::default();
			actions.push(Action::from_kdl(entry_node, &mut value_idx, system)?);
		}

		let mut mutators = Vec::new();
		for entry_node in node.query_all("mutator")? {
			let mut value_idx = ValueIdx::default();
			let id = entry_node.get_str(value_idx.next())?;
			let factory = system.get_mutator_factory(id)?;
			mutators.push(factory.from_kdl(entry_node, &mut value_idx, system)?);
		}

		Ok(Self {
			name,
			description,
			mutators,
			actions,
			criteria,
			// Generated data
			missing_selection_text: None,
		})
	}
}

#[derive(Clone, PartialEq)]
pub struct BoxedFeature(Arc<Feature>);
impl From<Feature> for BoxedFeature {
	fn from(feature: Feature) -> Self {
		Self(Arc::new(feature))
	}
}
impl BoxedFeature {
	pub fn inner(&self) -> &Feature {
		&*self.0
	}
}
impl std::fmt::Debug for BoxedFeature {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}
