pub use kdlize::{error::*, ext::*, AsKdl, FromKdl, NodeBuilder};

mod context;
pub use context::*;

pub type NodeReader<'doc> = kdlize::NodeReader<'doc, NodeContext>;

#[cfg(test)]
pub mod test_utils {
	use crate::kdl_ext::{AsKdl, FromKdl, NodeBuilder, NodeContext, NodeReader};

	macro_rules! assert_eq_fromkdl {
		($impl_ty:ty, $doc_str:expr, $expected_data:expr) => {
			let parsed: $impl_ty = from_doc(NODE_NAME, $doc_str, node_ctx(), from_kdl)?;
			assert_eq!(parsed, $expected_data);
		};
	}
	pub(crate) use assert_eq_fromkdl;

	macro_rules! assert_eq_askdl {
		($data:expr, $expected_doc:expr) => {
			let stringified = as_kdl($data).build(NODE_NAME).to_string();
			assert_eq!(stringified, raw_doc($expected_doc));
		};
	}
	pub(crate) use assert_eq_askdl;

	pub fn node_ctx() -> NodeContext {
		NodeContext::default()
	}

	pub fn from_kdl<'doc, T: FromKdl<NodeContext>>(mut node: NodeReader<'doc>) -> Result<T, T::Error> {
		T::from_kdl(&mut node)
	}

	pub fn as_kdl(data: &impl AsKdl) -> NodeBuilder {
		data.as_kdl()
	}

	pub fn raw_doc(str: &str) -> String {
		use trim_margin::MarginTrimmable;
		str.trim_margin().unwrap_or_else(|| str.to_owned())
	}

	pub fn from_doc<T, F>(name: &'static str, doc: &str, ctx: NodeContext, from_kdl: F) -> anyhow::Result<T>
	where
		F: Fn(NodeReader<'_>) -> anyhow::Result<T>,
	{
		let document = raw_doc(doc).parse::<kdl::KdlDocument>()?;
		let node = document
			.query(format!("scope() > {name}"))?
			.expect(&format!("missing {name} node"));
		from_kdl(NodeReader::new_child(node, ctx))
	}
}
