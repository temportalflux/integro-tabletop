use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use crate::system::core::SourceId;
use crate::system::dnd5e::SystemComponent;
use crate::{kdl_ext::NodeContext, utility::NotInList};
use derivative::Derivative;
use kdl::{KdlDocument, KdlValue};
use kdlize::ext::{DocumentExt, ValueExt};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

use super::{
	data::{description, item::armor, Rarity},
	BoxedMutator,
};

// Describes data which is used to generate variants of other components/objects.
#[derive(Clone, PartialEq, Debug)]
pub enum Generator {
	// Kdl generators apply a set of variants to blocks of a kdl document string,
	// thereby creating variations of that document which are parsed and interpretted by the system's component registry.
	Kdl(KdlGenerator),
	// Object iterators gather all known objects under a particular category (e.g. items or races/lineages),
	// and applies a set of mutating effects on a copy of each object, thereby creating variations of them.
	Object(ObjectGenerator),
}

#[derive(Clone, Derivative)]
#[derivative(PartialEq, Debug)]
pub struct KdlGenerator {
	id: SourceId,
	#[derivative(PartialEq = "ignore", Debug = "ignore")]
	base_doc: KdlDocument,
	base_str: String,
	variants: Vec<KdlVariant>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct KdlVariant(BTreeMap<String, KdlValue>);

#[derive(Clone, PartialEq, Debug)]
pub enum ObjectGenerator {
	// generates variants of items
	Item(ItemGenerator),
}

#[derive(Clone, PartialEq, Debug)]
pub struct ItemGenerator {
	id: SourceId,
	// the filter applied to search for the item objects
	filter: ItemFilter,
	variants: Vec<ItemVariant>,
}

// NOTE: always exclude generated objects
#[derive(Clone, PartialEq, Debug)]
pub struct ItemFilter {
	// Tags which must exist on the item for it to be a relevant base.
	tags: Vec<String>,
	// If provided, the item must be an equipment that provides armor and be one of the provided types.
	// Providing an empty set implies that all armor types are relevant.
	armor: Option<BTreeSet<armor::Kind>>,
	// If provided, the item must have a rarity in the set to be a relevant base.
	// If one of the entries in the set is `None`, then items which have no rarity are relevant.
	// If an empty set is provided, then all item rarities (including no rarity) are relevant.
	rarity: BTreeSet<Option<Rarity>>,
}

// Represents a variation that is applied to numerous base items.
#[derive(Clone, PartialEq, Debug)]
pub struct ItemVariant(Vec<ItemExtension>);

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
	formula: Option<ArmorFormulaExtension>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorFormulaExtension {
	base_bonus: Option<i32>,
}

kdlize::impl_kdl_node!(Generator, "generator");

impl SystemComponent for Generator {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"id": match &self {
				Self::Kdl(kdl) => kdl.id.unversioned().to_string(),
				Self::Object(ObjectGenerator::Item(item)) => item.id.unversioned().to_string(),
			},
			"kind": match &self {
				Self::Kdl(_) => "kdl",
				Self::Object(_) => "object",
			},
		})
	}
}

impl FromKdl<NodeContext> for Generator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = crate::kdl_ext::query_source_req(node)?;
		match node.next_str_req()? {
			"kdl" => {
				let mut base_doc = node.query_req("scope() > base")?.document_req()?.clone();
				base_doc.clear_fmt_recursive();
				let base_str = base_doc.to_string();
				let mut variants = node.query_all_t("scope() > variant")?;
				// prune out any variants with no entries
				variants.retain(|variant: &KdlVariant| !variant.0.is_empty());
				Ok(Generator::Kdl(KdlGenerator {
					id,
					base_doc,
					base_str,
					variants,
				}))
			}
			"object" => match node.next_str_req()? {
				"item" => {
					let filter = node.query_req_t("scope() > filter")?;
					let variants = node.query_all_t("scope() > variant")?;
					Ok(Generator::Object(ObjectGenerator::Item(ItemGenerator {
						id,
						filter,
						variants,
					})))
				}
				obj_kind => Err(NotInList(obj_kind.into(), vec!["item"]).into()),
			},
			kind => Err(NotInList(kind.into(), vec!["kdl", "object"]).into()),
		}
	}
}

impl AsKdl for Generator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Kdl(kdl) => {
				node.push_entry("kdl");
				node.push_child_opt_t("source", &kdl.id);
				// pushing the base means cloning all of the nodes in the document,
				// as children of a "base" node that we build
				node.push_child({
					let mut node = NodeBuilder::default();
					for child in kdl.base_doc.nodes() {
						node.push_child(child.clone());
					}
					node.build("base")
				});
				// pushing variants by delegating to the variant struct
				for variant in &kdl.variants {
					node.push_child_opt_t("variant", variant);
				}
				node
			}
			Self::Object(ObjectGenerator::Item(item)) => {
				node.push_entry("object");
				node.push_entry("item");
				node.push_child_opt_t("source", &item.id);
				node.push_child_t("filter", &item.filter);
				for variant in &item.variants {
					node.push_child_opt_t("variant", variant);
				}
				node
			}
		}
	}
}

impl FromKdl<NodeContext> for KdlVariant {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut entries = BTreeMap::default();
		for mut entry_node in node.query_all("scope() > entry")? {
			let key = entry_node.next_str_req()?;
			let value = entry_node.next_req()?.value().clone();
			entries.insert(key.to_owned(), value);
		}
		if entries.is_empty() {
			let key = node.next_str_req()?;
			let value = node.next_req()?.value().clone();
			entries.insert(key.to_owned(), value);
		}
		Ok(Self(entries))
	}
}

impl AsKdl for KdlVariant {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		let single_entry = self.0.len() == 1;
		let mut entry_iter = self.0.iter();
		if single_entry {
			let (key, value) = entry_iter.next().unwrap();
			node.push_entry(key.clone());
			node.push_entry(value.clone());
			return node;
		}

		for (key, value) in entry_iter {
			node.push_child(
				{
					let mut node = NodeBuilder::default();
					node.push_entry(key.clone());
					node.push_entry(value.clone());
					node
				}
				.build("entry"),
			);
		}

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
			node.push_child_t("tag", tag);
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
			node.push_child_t("extend", extension);
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
					node.push_child_t("section", section);
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
				if let Some(armor) = armor {
					node.push_child_t("armor", armor);
				}
				for mutator in mutators {
					node.push_child_t("mutator", mutator);
				}
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
			node.push_child_t("formula", formula);
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

#[cfg(test)]
mod test {
	use super::*;

	mod generator {
		use super::*;
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				core::{ModuleId, NodeRegistry},
				dnd5e::{
					data::DamageType,
					mutator::{AddDefense, Defense},
				},
			},
			utility::selector,
		};

		static NODE_NAME: &str = "generator";

		fn node_ctx() -> NodeContext {
			NodeContext::registry({
				let mut node_reg = NodeRegistry::default();
				node_reg.register_mutator::<AddDefense>();
				node_reg
			})
		}

		#[test]
		fn kdl_basic() -> anyhow::Result<()> {
			let base_str = "
				|item name=\"Belt of {TYPE} Strength\" {
				|    rarity \"{RARITY}\"
				|    tag \"Wonderous\"
				|    kind \"Equipment\" requires_attunement=true {
				|        minimum \"SCORE\"
				|    }
				|}
				|
			";
			let doc = "
				|generator \"kdl\" {
				|    source \"local://homebrew@dnd5e/items/generator.kdl\"
				|    base {
				|        item name=\"Belt of {TYPE} Strength\" {
				|            rarity \"{RARITY}\"
				|            tag \"Wonderous\"
				|            kind \"Equipment\" requires_attunement=true {
				|                minimum \"SCORE\"
				|            }
				|        }
				|    }
				|    variant \"TYPE\" \"Simple\"
				|    variant {
				|        entry \"RARITY\" \"Rare\"
				|        entry \"SCORE\" 21
				|        entry \"TYPE\" \"Huge\"
				|    }
				|}
			";
			let data = Generator::Kdl(KdlGenerator {
				id: SourceId {
					module: Some(ModuleId::Local {
						name: "homebrew".into(),
					}),
					system: Some("dnd5e".into()),
					path: "items/generator.kdl".into(),
					..Default::default()
				},
				base_doc: raw_doc(base_str)
					.parse::<KdlDocument>()
					.expect("failed to parse base kdl doc"),
				base_str: raw_doc(base_str),
				variants: [
					KdlVariant([("TYPE".into(), "Simple".into())].into()),
					KdlVariant(
						[
							("TYPE".into(), "Huge".into()),
							("RARITY".into(), "Rare".into()),
							("SCORE".into(), 21.into()),
						]
						.into(),
					),
				]
				.into(),
			});
			assert_eq_fromkdl!(Generator, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn item_simple() -> anyhow::Result<()> {
			let doc = "
				|generator \"object\" \"item\" {
				|    source \"local://homebrew@dnd5e/items/generator.kdl\"
				|    filter {
				|        tag \"Arrow\"
				|    }
				|    variant {
				|        extend \"name\" \"{name} +1\"
				|        extend \"rarity\" \"Rare\"
				|        extend \"description\" {
				|            section \"Does +1 extra damage (probably).\"
				|        }
				|    }
				|}
			";
			let data = Generator::Object(ObjectGenerator::Item(ItemGenerator {
				id: SourceId {
					module: Some(ModuleId::Local {
						name: "homebrew".into(),
					}),
					system: Some("dnd5e".into()),
					path: "items/generator.kdl".into(),
					..Default::default()
				},
				filter: ItemFilter {
					tags: ["Arrow".into()].into(),
					armor: None,
					rarity: [].into(),
				},
				variants: vec![ItemVariant(vec![
					ItemExtension::Name("{name} +1".into()),
					ItemExtension::Rarity(Some(Rarity::Rare)),
					ItemExtension::Description(vec![description::Section {
						content: description::SectionContent::Body("Does +1 extra damage (probably).".into()),
						..Default::default()
					}]),
				])],
			}));
			assert_eq_fromkdl!(Generator, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn item_equipment() -> anyhow::Result<()> {
			let doc = "
				|generator \"object\" \"item\" {
				|    source \"local://homebrew@dnd5e/items/generator.kdl\"
				|    filter {
				|        armor \"Medium\" \"Heavy\"
				|        rarity \"None\" \"Common\"
				|    }
				|    variant {
				|        extend \"name\" \"{name} Armor of Magic Resistance\"
				|        extend \"rarity\" \"Legendary\"
				|        extend \"description\" {
				|            section \"You have resistance to magic damage while you wear this armor.\"
				|        }
				|        extend \"equipment\" requires_attunement=true {
				|            armor {
				|                formula base_bonus=5
				|            }
				|            mutator \"add_defense\" \"Resistance\" (DamageType)\"Specific\" \"Psychic\"
				|        }
				|    }
				|}
			";
			let data = Generator::Object(ObjectGenerator::Item(ItemGenerator {
				id: SourceId {
					module: Some(ModuleId::Local {
						name: "homebrew".into(),
					}),
					system: Some("dnd5e".into()),
					path: "items/generator.kdl".into(),
					..Default::default()
				},
				filter: ItemFilter {
					tags: [].into(),
					armor: Some([armor::Kind::Medium, armor::Kind::Heavy].into()),
					rarity: [None, Some(Rarity::Common)].into(),
				},
				variants: vec![ItemVariant(vec![
					ItemExtension::Name("{name} Armor of Magic Resistance".into()),
					ItemExtension::Rarity(Some(Rarity::Legendary)),
					ItemExtension::Description(vec![description::Section {
						content: description::SectionContent::Body(
							"You have resistance to magic damage while you wear this armor.".into(),
						),
						..Default::default()
					}]),
					ItemExtension::Equipment {
						requires_attunement: Some(true),
						armor: Some(ArmorExtension {
							formula: Some(ArmorFormulaExtension { base_bonus: Some(5) }),
						}),
						mutators: [AddDefense {
							defense: Defense::Resistance,
							damage_type: Some(selector::Value::Specific(DamageType::Psychic)),
							context: None,
						}
						.into()]
						.into(),
					},
				])],
			}));
			assert_eq_fromkdl!(Generator, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
