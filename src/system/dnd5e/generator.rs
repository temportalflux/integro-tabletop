use crate::{kdl_ext::NodeContext, system::dnd5e::SystemComponent, utility::NotInList};

use kdlize::{AsKdl, FromKdl, NodeBuilder};

pub mod kdl;
pub mod object;
pub mod queue;

pub use kdl::Generator as KdlGenerator;
pub use object::item::Generator as ItemGenerator;
pub use object::Generator as ObjectGenerator;

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

kdlize::impl_kdl_node!(Generator, "generator");

impl SystemComponent for Generator {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"id": match &self {
				Self::Kdl(kdl) => kdl.id().unversioned().to_string(),
				Self::Object(object) => object.id().unversioned().to_string(),
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
		match node.next_str_req()? {
			"kdl" => Ok(Generator::Kdl(KdlGenerator::from_kdl(node)?)),
			"object" => Ok(Generator::Object(ObjectGenerator::from_kdl(node)?)),
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
				node += kdl.as_kdl();
				node
			}
			Self::Object(object) => {
				node.push_entry("object");
				node += object.as_kdl();
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod generator {
		use ::kdl::KdlDocument;
		use super::{
			kdl::KdlVariant,
			object::item::{ArmorExtension, ArmorFormulaExtension, ItemExtension, ItemFilter, ItemVariant},
			*,
		};
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				core::{ModuleId, NodeRegistry, SourceId},
				dnd5e::{
					data::{description, item::armor, DamageType, Rarity},
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
