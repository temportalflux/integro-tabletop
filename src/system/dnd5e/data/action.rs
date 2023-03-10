use super::condition::BoxedCondition;
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::{core::NodeRegistry, dnd5e::FromKDL},
};
use std::path::PathBuf;
use uuid::Uuid;

mod activation;
pub use activation::*;
mod attack;
pub use attack::*;
mod limited_uses;
pub use limited_uses::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Action {
	pub name: String,
	pub description: String,
	pub activation_kind: ActivationKind,
	pub attack: Option<Attack>,
	/// Dictates how many times this action can be used until it is reset.
	pub limited_uses: Option<LimitedUses>,
	/// Conditions applied when the action is used.
	pub apply_conditions: Vec<BoxedCondition>,
	// generated
	pub source: Option<ActionSource>,
}
impl FromKDL for Action {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.map(str::to_owned)
			.unwrap_or_default();
		let activation_kind = ActivationKind::from_kdl(
			node.query_req("activation")?,
			&mut ValueIdx::default(),
			node_reg,
		)?;
		let attack = match node.query("attack")? {
			None => None,
			Some(node) => Some(Attack::from_kdl(node, &mut ValueIdx::default(), node_reg)?),
		};
		let limited_uses = match node.query("limited_uses")? {
			None => None,
			Some(node) => Some(LimitedUses::from_kdl(
				node,
				&mut ValueIdx::default(),
				node_reg,
			)?),
		};
		// TODO: conditions applied on use
		let apply_conditions = Vec::new();
		Ok(Self {
			name,
			description,
			activation_kind,
			attack,
			limited_uses,
			apply_conditions,
			source: None,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActionSource {
	Item(Uuid),
	Feature(PathBuf),
}
