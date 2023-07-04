use crate::kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder};
use crate::system::{
	core::SourceId,
	dnd5e::data::{
		currency::Wallet,
		item::{weapon, Item},
	},
};
use crate::utility::NotInList;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum StartingEquipment {
	Currency(Wallet),
	SpecificItem(SourceId),
	CustomItem(Item),
	SelectItem(ItemFilter),
	Group {
		entries: Vec<StartingEquipment>,
		pick: Option<usize>,
	},
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ItemFilter {
	pub tags: Vec<String>,
	pub weapon: Option<WeaponFilter>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WeaponFilter {
	pub kind: Option<weapon::Kind>,
	pub has_melee: Option<bool>,
}

impl StartingEquipment {
	fn node_name(&self) -> &'static str {
		match self {
			Self::Currency(_) => "currency",
			Self::SpecificItem(_) | Self::CustomItem(_) | Self::SelectItem(_) => "item",
			Self::Group { .. } => "group",
		}
	}

	pub fn from_kdl_vec<'doc>(
		node: &mut crate::kdl_ext::NodeReader<'doc>,
	) -> anyhow::Result<Vec<Self>> {
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
			node.push_child_t(entry.node_name(), entry);
		}
		node
	}
}

impl FromKDL for StartingEquipment {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.name().value() {
			"currency" => Ok(Self::Currency(Wallet::from_kdl(node)?)),
			"group" => {
				let entries = StartingEquipment::from_kdl_vec(node)?;
				let pick = node.get_i64_opt("pick")?.map(|v| v as usize);
				Ok(Self::Group { entries, pick })
			}
			"item" => match node.next_str_req()? {
				"Specific" => {
					let id = node.next_str_req_t::<SourceId>()?;
					Ok(Self::SpecificItem(id.with_basis(node.id(), false)))
				}
				"Custom" => Ok(Self::CustomItem(Item::from_kdl(node)?)),
				"Select" => Ok(Self::SelectItem(ItemFilter::from_kdl(node)?)),
				kind => Err(NotInList(kind.into(), vec!["Specific", "Custom", "Select"]).into()),
			},
			name => {
				Err(NotInList(name.into(), vec!["item", "currency", "pick-one", "group"]).into())
			}
		}
	}
}
impl AsKdl for StartingEquipment {
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Currency(wallet) => wallet.as_kdl(),
			Self::SpecificItem(id) => NodeBuilder::default()
				.with_entry("Specific")
				.with_entry(id.to_string()),
			Self::CustomItem(item) => NodeBuilder::default()
				.with_entry("Custom")
				.with_extension(item.as_kdl()),
			Self::SelectItem(filter) => NodeBuilder::default()
				.with_entry("Select")
				.with_extension(filter.as_kdl()),
			Self::Group { entries, pick } => {
				let mut node = StartingEquipment::to_kdl_vec(entries);
				if let Some(amt) = pick {
					node.push_entry(("pick", *amt as i64));
				}
				node
			}
		}
	}
}

impl FromKDL for ItemFilter {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let tags = node
			.query_str_all("scope() > tag", 0)?
			.into_iter()
			.map(str::to_owned)
			.collect::<Vec<_>>();
		let weapon = node.query_opt_t::<WeaponFilter>("scope() > weapon")?;
		Ok(Self { tags, weapon })
	}
}
impl AsKdl for ItemFilter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for tag in &self.tags {
			node.push_child_t("tag", tag);
		}
		if let Some(weapon_filter) = &self.weapon {
			node.push_child_t("weapon", weapon_filter);
		}
		node
	}
}

impl FromKDL for WeaponFilter {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = match node.get_str_opt("kind")? {
			None => None,
			Some(str) => Some(weapon::Kind::from_str(str)?),
		};
		let has_melee = node.get_bool_opt("has_melee")?;
		Ok(Self { kind, has_melee })
	}
}
impl AsKdl for WeaponFilter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(kind) = &self.kind {
			node.push_entry(("kind", kind.to_string()));
		}
		if let Some(has_melee) = &self.has_melee {
			node.push_entry(("has_melee", *has_melee));
		}
		node
	}
}
