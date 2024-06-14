use crate::kdl_ext::NodeContext;
use crate::{
	system::{
		dnd5e::data::{
			currency::Wallet,
			item::{self, Item},
		},
		SourceId,
	},
	utility::NotInList,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub enum StartingEquipment {
	Currency(Wallet),
	IndirectItem(IndirectItem),
	SelectItem(item::Restriction),
	Group {
		entries: Vec<StartingEquipment>,
		pick: Option<usize>,
	},
}

impl StartingEquipment {
	fn node_name(&self) -> &'static str {
		match self {
			Self::Currency(_) => "currency",
			Self::IndirectItem(_) | Self::SelectItem(_) => "item",
			Self::Group { .. } => "group",
		}
	}

	pub fn from_kdl_vec<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Vec<Self>> {
		let mut entries = Vec::new();
		if let Some(children) = node.children() {
			for mut node in children {
				entries.push(Self::from_kdl(&mut node)?);
			}
		}
		Ok(entries)
	}

	pub fn to_kdl_vec(entries: &Vec<Self>) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for entry in entries {
			node.child((entry.node_name(), entry));
		}
		node
	}
}

impl FromKdl<NodeContext> for StartingEquipment {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.name().value() {
			"currency" => Ok(Self::Currency(Wallet::from_kdl(node)?)),
			"group" => {
				let entries = StartingEquipment::from_kdl_vec(node)?;
				let pick = node.get_i64_opt("pick")?.map(|v| v as usize);
				Ok(Self::Group { entries, pick })
			}
			"item" => match node.peak_str_req()? {
				"Specific" | "Custom" => Ok(Self::IndirectItem(IndirectItem::from_kdl(node)?)),
				"Select" => {
					let _ = node.next_str_req()?;
					Ok(Self::SelectItem(item::Restriction::from_kdl(node)?))
				}
				kind => Err(NotInList(kind.into(), vec!["Specific", "Custom", "Select"]).into()),
			},
			name => Err(NotInList(name.into(), vec!["item", "currency", "group"]).into()),
		}
	}
}
impl AsKdl for StartingEquipment {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Currency(wallet) => wallet.as_kdl(),
			Self::IndirectItem(indirect) => indirect.as_kdl(),
			Self::SelectItem(filter) => {
				let mut node = NodeBuilder::default();
				node.entry("Select");
				node += filter.as_kdl();
				node
			}
			Self::Group { entries, pick } => {
				let mut node = StartingEquipment::to_kdl_vec(entries);
				node.entry(("pick", pick.as_ref().map(|i| *i as i64)));
				node
			}
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum IndirectItem {
	Specific(SourceId, usize),
	Custom(Item),
}
impl FromKdl<NodeContext> for IndirectItem {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"Specific" => {
				let id = node.next_str_req_t::<SourceId>()?;
				let id = id.with_relative_basis(node.context().id(), false);
				let count = node.next_i64_opt()?.unwrap_or(1) as usize;
				Ok(Self::Specific(id, count))
			}
			"Custom" => Ok(Self::Custom(Item::from_kdl(node)?)),
			kind => Err(NotInList(kind.into(), vec!["Specific", "Custom"]).into()),
		}
	}
}
impl AsKdl for IndirectItem {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Specific(id, count) => {
				let mut node = NodeBuilder::default().with_entry("Specific");
				let kdl = id.as_kdl();
				if !kdl.is_empty() {
					node += kdl;
				}
				if *count > 1 {
					node.entry(*count as i64);
				}
				node
			}
			Self::Custom(item) => {
				let mut node = NodeBuilder::default();
				node.entry("Custom");
				node += item.as_kdl();
				node
			}
		}
	}
}
