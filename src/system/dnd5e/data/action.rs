use super::{description, IndirectCondition};
use crate::kdl_ext::{DocumentExt, FromKDL, NodeExt};
use std::{path::{PathBuf, Path}, borrow::Cow};
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
	pub description: description::Info,
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
		let description = description::Info::from_kdl_all(node, ctx)?;
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
impl ActionSource {
	pub fn as_path<'this>(&'this self, inventory: &super::item::Inventory) -> Cow<'this, Path> {
		match self {
			Self::Feature(path) => Cow::Borrowed(path.as_path()),
			Self::Item(id) => {
				let base = PathBuf::new().join("Equipment");
				let owned = match inventory.get_item(id) {
					Some(item) => base.join(&item.name),
					None => base.join("Unknown Item"),
				};
				Cow::Owned(owned)
			}
		}
	}
}
