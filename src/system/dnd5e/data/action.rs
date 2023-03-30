use super::IndirectCondition;
use crate::kdl_ext::{DocumentExt, FromKDL, NodeExt};
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
	pub description: Option<String>,
	pub short_desc: Option<String>,
	pub activation_kind: ActivationKind,
	pub attack: Option<Attack>,
	/// Dictates how many times this action can be used until it is reset.
	pub limited_uses: Option<LimitedUses>,
	/// Conditions applied when the action is used.
	pub conditions_to_apply: Vec<IndirectCondition>,
	// generated
	pub source: Option<ActionSource>,
}

impl Action {
	pub fn set_data_path(&self, parent: &std::path::Path) {
		if let Some(uses) = &self.limited_uses {
			uses.set_data_path(parent);
		}
	}
}

impl FromKDL for Action {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.map(str::to_owned);
		let short_desc = node
			.query_str_opt("scope() > description > short", 0)?
			.map(str::to_owned);
		let activation_kind = ActivationKind::from_kdl(
			node.query_req("scope() > activation")?,
			&mut ctx.next_node(),
		)?;

		let attack = match node.query("scope() > attack")? {
			None => None,
			Some(node) => Some(Attack::from_kdl(node, &mut ctx.next_node())?),
		};
		let limited_uses = match node.query("scope() > limited_uses")? {
			None => None,
			Some(node) => Some(LimitedUses::from_kdl(node, &mut ctx.next_node())?),
		};

		let mut conditions_to_apply = Vec::new();
		for node in node.query_all("scope() > condition")? {
			conditions_to_apply.push(IndirectCondition::from_kdl(node, &mut ctx.next_node())?);
		}

		Ok(Self {
			name,
			description,
			short_desc,
			activation_kind,
			attack,
			limited_uses,
			conditions_to_apply,
			source: None,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ActionSource {
	Item(Uuid),
	Feature(PathBuf),
}
