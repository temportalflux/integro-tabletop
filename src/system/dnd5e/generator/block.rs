use crate::{kdl_ext::NodeContext, system::core::SourceId, utility::{PinFuture, SystemObjectList}};
use database::Transaction;
use derivative::Derivative;
use kdl::{KdlDocument, KdlValue};
use kdlize::{AsKdl, FromKdl, NodeBuilder, OmitIfEmpty};
use std::collections::BTreeMap;

/// Kdl generators apply a set of variants to blocks of a kdl document string,
/// thereby creating variations of that document which are parsed and interpretted by the system's component registry.
#[derive(Clone, Derivative)]
#[derivative(PartialEq, Debug)]
pub struct BlockGenerator {
	pub(super) id: SourceId,
	#[derivative(PartialEq = "ignore", Debug = "ignore")]
	pub(super) base_doc: KdlDocument,
	pub(super) base_str: String,
	pub(super) variants: Vec<VariantEntry>,
}

kdlize::impl_kdl_node!(BlockGenerator, "block");
crate::impl_trait_eq!(BlockGenerator);

impl crate::utility::Generator for BlockGenerator {
	fn source_id(&self) -> &SourceId { &self.id }
	fn execute(&self, context: &NodeContext, transaction: &Transaction) -> PinFuture<anyhow::Result<SystemObjectList>> {
		Box::pin(async move {
			let mut output = SystemObjectList::default();
			Ok(output) as anyhow::Result<SystemObjectList>
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct VariantEntry(pub(super) BTreeMap<String, KdlValue>);

impl BlockGenerator {
	pub fn id(&self) -> &SourceId {
		&self.id
	}
}

impl FromKdl<NodeContext> for BlockGenerator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = crate::kdl_ext::query_source_req(node)?;
		let mut base_doc = node.query_req("scope() > base")?.document_req()?.clone();
		base_doc.clear_fmt_recursive();
		let base_str = base_doc.to_string();
		let mut variants = node.query_all_t("scope() > variant")?;
		// prune out any variants with no entries
		variants.retain(|variant: &VariantEntry| !variant.0.is_empty());
		Ok(Self {
			id,
			base_doc,
			base_str,
			variants,
		})
	}
}

impl AsKdl for BlockGenerator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_child_t(("source", &self.id, OmitIfEmpty));
		// pushing the base means cloning all of the nodes in the document,
		// as children of a "base" node that we build
		node.push_child({
			let mut node = NodeBuilder::default();
			for child in self.base_doc.nodes() {
				node.push_child(child.clone());
			}
			node.build("base")
		});
		// pushing variants by delegating to the variant struct
		node.push_children_t(("variant", self.variants.iter(), OmitIfEmpty));
		node
	}
}

impl FromKdl<NodeContext> for VariantEntry {
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

impl AsKdl for VariantEntry {
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

#[cfg(test)]
mod test {
	use super::*;

	mod generator {
		use super::*;
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::core::{ModuleId, NodeRegistry, SourceId},
			utility::GenericGenerator,
		};
		use ::kdl::KdlDocument;

		static NODE_NAME: &str = "generator";

		fn node_ctx() -> NodeContext {
			NodeContext::registry({
				let mut node_reg = NodeRegistry::default();
				node_reg.register_generator::<BlockGenerator>();
				node_reg
			})
		}

		#[test]
		fn block_basic() -> anyhow::Result<()> {
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
				|generator \"block\" {
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
			let data = BlockGenerator {
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
					VariantEntry([("TYPE".into(), "Simple".into())].into()),
					VariantEntry(
						[
							("TYPE".into(), "Huge".into()),
							("RARITY".into(), "Rare".into()),
							("SCORE".into(), 21.into()),
						]
						.into(),
					),
				]
				.into(),
			};
			let generator = GenericGenerator::from(data);
			assert_eq_fromkdl!(GenericGenerator, doc, generator);
			assert_eq_askdl!(&generator, doc);
			Ok(())
		}
	}
}
