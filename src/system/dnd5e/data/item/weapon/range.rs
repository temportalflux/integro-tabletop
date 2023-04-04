use crate::kdl_ext::{FromKDL, NodeContext, NodeExt};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Range {
	pub short_range: u32,
	pub long_range: u32,
	pub requires_ammunition: bool,
	pub requires_loading: bool,
}

impl FromKDL for Range {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let short_range = node.get_i64_req(ctx.consume_idx())? as u32;
		let long_range = node.get_i64_req(ctx.consume_idx())? as u32;
		let requires_ammunition = node.query("scope() > ammunition")?.is_some();
		let requires_loading = node.query("scope() > loading")?.is_some();
		Ok(Self {
			short_range,
			long_range,
			requires_ammunition,
			requires_loading,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::kdl_ext::NodeContext;

	fn from_doc(doc: &str) -> anyhow::Result<Range> {
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document
			.query("scope() > range")?
			.expect("missing range node");
		Range::from_kdl(node, &mut NodeContext::default())
	}

	#[test]
	fn base() -> anyhow::Result<()> {
		let doc = "range 20 60";
		let expected = Range {
			short_range: 20,
			long_range: 60,
			requires_ammunition: false,
			requires_loading: false,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}

	#[test]
	fn ammunition() -> anyhow::Result<()> {
		let doc = "range 25 100 {
			ammunition
		}";
		let expected = Range {
			short_range: 25,
			long_range: 100,
			requires_ammunition: true,
			requires_loading: false,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}

	#[test]
	fn loading() -> anyhow::Result<()> {
		let doc = "range 25 100 {
			loading
		}";
		let expected = Range {
			short_range: 25,
			long_range: 100,
			requires_ammunition: false,
			requires_loading: true,
		};
		assert_eq!(from_doc(doc)?, expected);
		Ok(())
	}
}
