use super::Condition;
use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::core::SourceId,
};
use anyhow::Context;
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub enum IndirectCondition {
	Id(SourceId),
	Custom(Condition),
}

impl FromKDL for IndirectCondition {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Custom" => {
				// this is a custom condition node, parse it as a condition struct
				let condition = Condition::from_kdl(node, ctx)?;
				Ok(Self::Custom(condition))
			}
			source_id_str => {
				let mut source_id = SourceId::from_str(source_id_str).with_context(|| {
					format!("Expected {source_id_str:?} to either be the value \"Custom\" or a valid SourceId.")
				})?;
				source_id.set_basis(ctx.id(), false);
				Ok(Self::Id(source_id))
			}
		}
	}
}
// TODO AsKdl: tests for IndirectCondition
impl AsKdl for IndirectCondition {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Id(id) => node.with_entry(id.to_string()),
			Self::Custom(condition) => {
				let mut node = node.with_entry("Custom");
				node += condition.as_kdl();
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	mod from_kdl {
		use super::*;
		use crate::kdl_ext::NodeContext;

		fn from_doc(doc: &str) -> anyhow::Result<IndirectCondition> {
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > condition")?
				.expect("missing condition node");
			IndirectCondition::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn id() -> anyhow::Result<()> {
			let doc = "condition \"condition/invisible.kdl\"";
			let expected = IndirectCondition::Id(SourceId {
				path: "condition/invisible.kdl".into(),
				..Default::default()
			});
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn custom() -> anyhow::Result<()> {
			let doc = "condition \"Custom\" name=\"Slippery\"";
			let expected = IndirectCondition::Custom(Condition {
				name: "Slippery".into(),
				..Default::default()
			});
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
