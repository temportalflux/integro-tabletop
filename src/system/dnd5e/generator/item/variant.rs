use std::str::FromStr;

use crate::{
	kdl_ext::NodeContext,
	system::dnd5e::{
		data::{
			description,
			item::{self, Item},
			Rarity,
		},
		BoxedMutator,
	},
	utility::NotInList,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

mod armor;
pub use armor::*;
mod attunement;
pub use attunement::*;

// Represents a variation that is applied to numerous base items.
#[derive(Clone, PartialEq, Debug)]
pub struct Variant {
	pub name: String,
	pub extensions: Vec<Extension>,
}

// Represents an operation applied to an item when making a variant.
#[derive(Clone, PartialEq, Debug)]
pub enum Extension {
	// Renames the copy, formatting the provided string to replace any `{name}` substrings with the original name.
	Name(String),
	// Sets the provided rarity on the item.
	Rarity(Option<Rarity>),
	// Adds sections to the item's description.
	Description(Vec<description::Section>),
	// If the base is not already a piece of equipment, it is populated with default equipment data.
	// Then the following extensions are applied to the equipment data.
	Equipment {
		attunement: Option<AttunementExtension>,
		armor: Option<ArmorExtension>,
		// mutators to append to the equipment data
		mutators: Vec<BoxedMutator>,
	},
}

impl Variant {
	pub fn apply_to(&self, item: &mut Item) -> anyhow::Result<()> {
		for extension in &self.extensions {
			extension.apply_to(item)?;
		}
		Ok(())
	}
}

impl Extension {
	fn apply_to(&self, item: &mut Item) -> anyhow::Result<()> {
		match self {
			Self::Name(name_format) => {
				item.name = name_format.replace("{name}", &item.name);
			}
			Self::Rarity(new_rarity) => {
				item.rarity = new_rarity.clone();
			}
			Self::Description(sections_to_append) => {
				item.description.sections.extend(sections_to_append.clone());
			}
			Self::Equipment {
				attunement,
				armor,
				mutators,
			} => {
				let item::Kind::Equipment(equipment) = &mut item.kind else {
					return Ok(());
				};
				if let Some(attunement_ext) = attunement {
					attunement_ext.apply_to(equipment)?;
				}
				if let Some(armor_ext) = armor {
					armor_ext.apply_to(equipment)?;
				}
				equipment.mutators.extend(mutators.clone());
			}
		}
		Ok(())
	}
}

impl FromKdl<NodeContext> for Variant {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let name = node.next_str_req()?.to_owned();
		let extensions = node.query_all_t("scope() > extend")?;
		Ok(Self { name, extensions })
	}
}

impl AsKdl for Variant {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.name.as_str());
		for extension in &self.extensions {
			node.child(("extend", extension));
		}
		node
	}
}

impl FromKdl<NodeContext> for Extension {
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
				let attunement = node.query_opt_t("scope() > attunement")?;
				let mutators = node.query_all_t("scope() > mutator")?;
				let armor = node.query_opt_t("scope() > armor")?;
				Ok(Self::Equipment {
					attunement,
					armor,
					mutators,
				})
			}
			kind => Err(NotInList(kind.into(), vec!["name", "rarity", "description", "equipment"]).into()),
		}
	}
}

impl AsKdl for Extension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Name(name) => {
				node.entry("name");
				node.entry(name.clone());
				node
			}
			Self::Rarity(rarity) => {
				node.entry("rarity");
				node.entry(rarity.as_ref().map(Rarity::to_string).unwrap_or("None".to_owned()));
				node
			}
			Self::Description(sections) => {
				node.entry("description");
				for section in sections {
					node.child(("section", section));
				}
				node
			}
			Self::Equipment {
				attunement,
				armor,
				mutators,
			} => {
				node.entry("equipment");
				node.child(("attunement", attunement));
				node.child(("armor", armor));
				node.children(("mutator", mutators.iter()));
				node
			}
		}
	}
}
