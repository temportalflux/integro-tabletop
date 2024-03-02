use crate::{
	kdl_ext::NodeContext,
	system::{
		core::SourceId,
		dnd5e::{
			data::{description, item::armor, Rarity},
			BoxedMutator,
		},
	},
	utility::NotInList,
};

use kdlize::{
	ext::{DocumentExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder, OmitIfEmpty,
};
use std::{collections::BTreeSet, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct Generator {
	pub(in super::super) id: SourceId,
	// the filter applied to search for the item objects
	pub(in super::super) filter: ItemFilter,
	pub(in super::super) variants: Vec<ItemVariant>,
}

// NOTE: always exclude generated objects
#[derive(Clone, PartialEq, Debug)]
pub struct ItemFilter {
	// Tags which must exist on the item for it to be a relevant base.
	pub(in super::super) tags: Vec<String>,
	// If provided, the item must be an equipment that provides armor and be one of the provided types.
	// Providing an empty set implies that all armor types are relevant.
	pub(in super::super) armor: Option<BTreeSet<armor::Kind>>,
	// If provided, the item must have a rarity in the set to be a relevant base.
	// If one of the entries in the set is `None`, then items which have no rarity are relevant.
	// If an empty set is provided, then all item rarities (including no rarity) are relevant.
	pub(in super::super) rarity: BTreeSet<Option<Rarity>>,
}

// Represents a variation that is applied to numerous base items.
#[derive(Clone, PartialEq, Debug)]
pub struct ItemVariant(pub(in super::super) Vec<ItemExtension>);

// Represents an operation applied to an item when making a variant.
#[derive(Clone, PartialEq, Debug)]
pub enum ItemExtension {
	// Renames the copy, formatting the provided string to replace any `{name}` substrings with the original name.
	Name(String),
	// Sets the provided rarity on the item.
	Rarity(Option<Rarity>),
	// Adds sections to the item's description.
	Description(Vec<description::Section>),
	// If the base is not already a piece of equipment, it is populated with default equipment data.
	// Then the following extensions are applied to the equipment data.
	Equipment {
		requires_attunement: Option<bool>,
		armor: Option<ArmorExtension>,
		// mutators to append to the equipment data
		mutators: Vec<BoxedMutator>,
	},
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorExtension {
	pub(in super::super) formula: Option<ArmorFormulaExtension>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorFormulaExtension {
	pub(in super::super) base_bonus: Option<i32>,
}

impl Generator {
	pub fn id(&self) -> &SourceId {
		&self.id
	}
}

impl FromKdl<NodeContext> for Generator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = crate::kdl_ext::query_source_req(node)?;
		let filter = node.query_req_t("scope() > filter")?;
		let variants = node.query_all_t("scope() > variant")?;
		Ok(Self { id, filter, variants })
	}
}

impl AsKdl for Generator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_child_t(("source", &self.id, OmitIfEmpty));
		node.push_child_t(("filter", &self.filter));
		node.push_children_t(("variant", self.variants.iter(), OmitIfEmpty));
		node
	}
}

impl FromKdl<NodeContext> for ItemFilter {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let tags = node.query_str_all("scope() > tag", 0)?;
		let tags = tags.into_iter().map(str::to_owned).collect();
		let armor = match node.query_opt("scope() > armor")? {
			None => None,
			Some(node) => {
				let mut kinds = BTreeSet::new();
				for entry in node.entries() {
					match entry.as_str_req()? {
						"Any" => {}
						value => {
							kinds.insert(armor::Kind::from_str(value)?);
						}
					}
				}
				Some(kinds)
			}
		};
		let rarity = match node.query_opt("scope() > rarity")? {
			None => BTreeSet::new(),
			Some(node) => {
				let mut rarities = BTreeSet::new();
				for entry in node.entries() {
					rarities.insert(match entry.as_str_req()? {
						"None" => None,
						value => Some(Rarity::from_str(value)?),
					});
				}
				rarities
			}
		};
		Ok(Self { tags, armor, rarity })
	}
}

impl AsKdl for ItemFilter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		for tag in &self.tags {
			node.push_child_t(("tag", tag));
		}

		if let Some(armor_kinds) = &self.armor {
			let mut node_armor = NodeBuilder::default();
			if armor_kinds.is_empty() {
				node_armor.push_entry("Any");
			} else {
				for kind in armor_kinds {
					node_armor.push_entry(kind.to_string());
				}
			}
			node.push_child(node_armor.build("armor"));
		}

		if !self.rarity.is_empty() {
			let mut node_rarity = NodeBuilder::default();
			for rarity in &self.rarity {
				node_rarity.push_entry(match rarity {
					None => "None".to_owned(),
					Some(rarity) => rarity.to_string(),
				});
			}
			node.push_child(node_rarity.build("rarity"));
		}

		node
	}
}

impl FromKdl<NodeContext> for ItemVariant {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(Self(node.query_all_t("scope() > extend")?))
	}
}

impl AsKdl for ItemVariant {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for extension in &self.0 {
			node.push_child_t(("extend", extension));
		}
		node
	}
}

impl FromKdl<NodeContext> for ItemExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"name" => Ok(Self::Name(node.next_str_req()?.to_owned())),
			"rarity" => Ok(Self::Rarity(match node.next_str_req()? {
				"None" => None,
				value => Some(Rarity::from_str(value)?),
			})),
			"description" => Ok(Self::Description(node.query_all_t("scope() > section")?)),
			"equipment" => {
				let requires_attunement = node.get_bool_opt("requires_attunement")?;
				let mutators = node.query_all_t("scope() > mutator")?;
				let armor = node.query_opt_t("scope() > armor")?;
				Ok(Self::Equipment {
					requires_attunement,
					armor,
					mutators,
				})
			}
			kind => Err(NotInList(kind.into(), vec!["name", "rarity", "description", "equipment"]).into()),
		}
	}
}

impl AsKdl for ItemExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Name(name) => {
				node.push_entry("name");
				node.push_entry(name.clone());
				node
			}
			Self::Rarity(rarity) => {
				node.push_entry("rarity");
				node.push_entry(rarity.as_ref().map(Rarity::to_string).unwrap_or("None".to_owned()));
				node
			}
			Self::Description(sections) => {
				node.push_entry("description");
				for section in sections {
					node.push_child_t(("section", section));
				}
				node
			}
			Self::Equipment {
				requires_attunement,
				armor,
				mutators,
			} => {
				node.push_entry("equipment");
				if let Some(required) = requires_attunement {
					node.push_entry(("requires_attunement", *required));
				}
				node.push_child_t(("armor", armor));
				node.push_children_t(("mutator", mutators.iter()));
				node
			}
		}
	}
}

impl FromKdl<NodeContext> for ArmorExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let formula = node.query_opt_t("scope() > formula")?;
		Ok(Self { formula })
	}
}

impl AsKdl for ArmorExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(formula) = &self.formula {
			node.push_child_t(("formula", formula));
		}
		node
	}
}

impl FromKdl<NodeContext> for ArmorFormulaExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let base_bonus = node.get_i64_opt("base_bonus")?.map(|num| num as i32);
		Ok(Self { base_bonus })
	}
}

impl AsKdl for ArmorFormulaExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(base_bonus) = &self.base_bonus {
			node.push_entry(("base_bonus", *base_bonus as i64));
		}
		node
	}
}
