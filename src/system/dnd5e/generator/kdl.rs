use crate::{kdl_ext::NodeContext, system::core::SourceId};
use derivative::Derivative;
use kdl::{KdlDocument, KdlValue};
use kdlize::{AsKdl, FromKdl, NodeBuilder, OmitIfEmpty};
use std::collections::BTreeMap;

#[derive(Clone, Derivative)]
#[derivative(PartialEq, Debug)]
pub struct Generator {
	pub(super) id: SourceId,
	#[derivative(PartialEq = "ignore", Debug = "ignore")]
	pub(super) base_doc: KdlDocument,
	pub(super) base_str: String,
	pub(super) variants: Vec<KdlVariant>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct KdlVariant(pub(super) BTreeMap<String, KdlValue>);

impl Generator {
	pub fn id(&self) -> &SourceId {
		&self.id
	}
}

impl FromKdl<NodeContext> for Generator {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = crate::kdl_ext::query_source_req(node)?;
		let mut base_doc = node.query_req("scope() > base")?.document_req()?.clone();
		base_doc.clear_fmt_recursive();
		let base_str = base_doc.to_string();
		let mut variants = node.query_all_t("scope() > variant")?;
		// prune out any variants with no entries
		variants.retain(|variant: &KdlVariant| !variant.0.is_empty());
		Ok(Self {
			id,
			base_doc,
			base_str,
			variants,
		})
	}
}

impl AsKdl for Generator {
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
