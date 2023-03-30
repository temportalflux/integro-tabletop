use super::character::Character;
use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{BoxedCriteria, BoxedMutator, DnD5e, SystemComponent},
	},
	utility::MutatorGroup,
};
use anyhow::Context;
use std::{path::Path, str::FromStr};

/// A state a character may be subject to until it is removed.
/// Some conditions are automatically cleared on the next long rest, and any conditions may be manually cleared.
/// Conditions contain a set of mutators and an optional criteria that, if met, applies those mutators.
#[derive(Clone, PartialEq, Debug)]
pub struct Condition {
	pub source_id: Option<SourceId>,
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,
}

crate::impl_kdl_node!(Condition, "condition");

impl MutatorGroup for Condition {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(&self.name);
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
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
	}
}

impl SystemComponent for Condition {
	type System = DnD5e;

	fn add_component(mut self, source_id: SourceId, system: &mut Self::System) {
		self.source_id = Some(source_id.clone());
		system.conditions.insert(source_id, self);
	}
}

impl FromKDL for Condition {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let criteria = match node.query("scope() > criteria")? {
			None => None,
			Some(entry_node) => {
				Some(ctx.parse_evaluator::<Character, Result<(), String>>(entry_node)?)
			}
		};

		Ok(Self {
			source_id: None,
			name,
			description,
			mutators,
			criteria,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum IndirectCondition {
	Id(SourceId),
	Custom(Condition),
}

impl IndirectCondition {
	/// Returns a reference to the underlying condition.
	/// If self is an Id, the value returned is retrieved from the system (if it exists).
	pub fn resolve<'a>(&'a self, system: &'a DnD5e) -> Option<&'a Condition> {
		match self {
			Self::Custom(value) => Some(value),
			Self::Id(id) => system.conditions.get(id),
		}
	}
}

impl FromKDL for IndirectCondition {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Custom" => {
				// this is a custom condition node, parse it as a condition struct
				let condition = Condition::from_kdl(node, ctx)?;
				Ok(Self::Custom(condition))
			}
			source_id_str => {
				let mut source_id = SourceId::from_str(source_id_str).with_context(|| {
					format!("Expected {source_id_str:?} to either be the value \"Custom\" or a valid SourceId.")
				})?;
				source_id.set_basis(ctx.id());
				Ok(Self::Id(source_id))
			}
		}
	}
}
