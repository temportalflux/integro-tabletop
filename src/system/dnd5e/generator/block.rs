use crate::{
	kdl_ext::NodeContext,
	system::{generator::SystemObjectList, SourceId},
	utility::PinFutureLifetimeNoSend,
};
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
	pub(super) short_id: String,
	#[derivative(PartialEq = "ignore", Debug = "ignore")]
	pub(super) base_doc: KdlDocument,
	pub(super) base_str: String,
	pub(super) variants: Vec<VariantEntry>,
}

kdlize::impl_kdl_node!(BlockGenerator, "block");
crate::impl_trait_eq!(BlockGenerator);

impl crate::system::Generator for BlockGenerator {
	fn source_id(&self) -> &SourceId {
		&self.id
	}

	fn short_id(&self) -> &String {
		&self.short_id
	}

	fn execute<'this>(
		&'this self,
		args: crate::system::generator::Args<'this>,
	) -> PinFutureLifetimeNoSend<'this, anyhow::Result<SystemObjectList>> {
		Box::pin(async move {
			let mut output = SystemObjectList::new(self, args.system.node());

			for variant_args in &self.variants {
				let mut variant_doc_str = self.base_str.clone();
				for (key, value) in &variant_args.entries {
					let value_string = value.to_string();
					// kdl wraps string values with double-quotes when formatting the value as a string.
					let value_str = value_string.trim_matches('"');
					variant_doc_str = variant_doc_str.replace(&format!("{{{key}}}"), value_str);
					variant_doc_str = variant_doc_str.replace(&format!("\"{key}\""), value_str);
				}
				let mut document = variant_doc_str.parse::<kdl::KdlDocument>()?;
				document.fmt();
				if document.nodes().len() > 1 {
					continue;
				}
				let Some(node) = document.nodes().first() else { continue };

				let mut source_id = self.source_id().clone();
				source_id.variant = Some(output.variant_id(variant_args.name.clone()));
				let metadata = args.system.parse_metadata(node, &source_id)?;
				let record = crate::database::Entry {
					id: source_id.to_string(),
					module: source_id.module.as_ref().unwrap().to_string(),
					system: source_id.system.clone().unwrap(),
					category: node.name().value().to_owned(),
					version: source_id.version.clone(),
					metadata,
					kdl: node.to_string(),
					file_id: None,
					generator_id: Some(self.source_id().to_string()),
					generated: 1,
				};
				output.insert(variant_args.name.clone(), record);
			}

			Ok(output) as anyhow::Result<SystemObjectList>
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct VariantEntry {
	pub(super) name: String,
	pub(super) entries: BTreeMap<String, KdlValue>,
}

impl BlockGenerator {
	pub fn id(&self) -> &SourceId {
		&self.id
	}
}

impl FromKdl<NodeContext> for BlockGenerator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = crate::kdl_ext::query_source_req(node)?;
		let short_id = node.next_str_req()?.to_owned();
		let mut base_doc = node.query_req("scope() > base")?.document_req()?.clone();
		base_doc.clear_fmt_recursive();
		let base_str = base_doc.to_string();
		let mut variants = node.query_all_t("scope() > variant")?;
		// prune out any variants with no entries
		variants.retain(|variant: &VariantEntry| !variant.entries.is_empty());
		Ok(Self {
			id,
			short_id,
			base_doc,
			base_str,
			variants,
		})
	}
}

impl AsKdl for BlockGenerator {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.short_id.as_str());
		node.child(("source", &self.id, OmitIfEmpty));
		// pushing the base means cloning all of the nodes in the document,
		// as children of a "base" node that we build
		node.child({
			let mut node = NodeBuilder::default();
			for child in self.base_doc.nodes() {
				node.child(child.clone());
			}
			node.build("base")
		});
		// pushing variants by delegating to the variant struct
		node.children(("variant", self.variants.iter(), OmitIfEmpty));
		node
	}
}

impl FromKdl<NodeContext> for VariantEntry {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut entries = BTreeMap::default();
		let name = node.next_str_req()?.to_owned();
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
		Ok(Self { name, entries })
	}
}

impl AsKdl for VariantEntry {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.entry(self.name.as_str());

		let single_entry = self.entries.len() == 1;
		let mut entry_iter = self.entries.iter();
		if single_entry {
			let (key, value) = entry_iter.next().unwrap();
			node.entry(key.clone());
			node.entry(value.clone());
			return node;
		}

		for (key, value) in entry_iter {
			node.child(
				{
					let mut node = NodeBuilder::default();
					node.entry(key.clone());
					node.entry(value.clone());
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
			system::{generator, generics, ModuleId, SourceId},
		};
		use ::kdl::KdlDocument;

		static NODE_NAME: &str = "generator";

		fn node_ctx() -> NodeContext {
			NodeContext::registry({
				let mut node_reg = generics::Registry::default();
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
				|    kind \"Equipment\" {
				|        attunement required=true
				|        minimum \"SCORE\"
				|    }
				|}
				|
			";
			let doc = "
				|generator \"block\" \"test\" {
				|    source \"local://homebrew@dnd5e/items/generator.kdl\"
				|    base {
				|        item name=\"Belt of {TYPE} Strength\" {
				|            rarity \"{RARITY}\"
				|            tag \"Wonderous\"
				|            kind \"Equipment\" {
				|                attunement required=true
				|                minimum \"SCORE\"
				|            }
				|        }
				|    }
				|    variant \"1\" \"TYPE\" \"Simple\"
				|    variant \"2\" {
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
				short_id: "test".to_owned(),
				base_doc: raw_doc(base_str)
					.parse::<KdlDocument>()
					.expect("failed to parse base kdl doc"),
				base_str: raw_doc(base_str),
				variants: [
					VariantEntry {
						name: "1".into(),
						entries: [("TYPE".into(), "Simple".into())].into(),
					},
					VariantEntry {
						name: "2".into(),
						entries: [
							("TYPE".into(), "Huge".into()),
							("RARITY".into(), "Rare".into()),
							("SCORE".into(), 21.into()),
						]
						.into(),
					},
				]
				.into(),
			};
			let generator = generator::Generic::from(data);
			assert_eq_fromkdl!(generator::Generic, doc, generator);
			assert_eq_askdl!(&generator, doc);
			Ok(())
		}
	}
}
