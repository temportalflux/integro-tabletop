use crate::kdl_ext::NodeContext;
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Range {
	pub short_range: u32,
	pub long_range: u32,
	pub requires_ammunition: bool,
	pub requires_loading: bool,
}

impl FromKdl<NodeContext> for Range {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let short_range = node.next_i64_req()? as u32;
		let long_range = node.next_i64_req()? as u32;
		let requires_ammunition = node.query_opt("scope() > ammunition")?.is_some();
		let requires_loading = node.query_opt("scope() > loading")?.is_some();
		Ok(Self { short_range, long_range, requires_ammunition, requires_loading })
	}
}

impl AsKdl for Range {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.short_range as i64);
		node.entry(self.long_range as i64);
		if self.requires_ammunition {
			node.child(NodeBuilder::default().build("ammunition"));
		}
		if self.requires_loading {
			node.child(NodeBuilder::default().build("loading"));
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "range";

		#[test]
		fn base() -> anyhow::Result<()> {
			let doc = "range 20 60";
			let data = Range { short_range: 20, long_range: 60, requires_ammunition: false, requires_loading: false };
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn ammunition() -> anyhow::Result<()> {
			let doc = "
				|range 25 100 {
				|    ammunition
				|}
			";
			let data = Range { short_range: 25, long_range: 100, requires_ammunition: true, requires_loading: false };
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn loading() -> anyhow::Result<()> {
			let doc = "
				|range 25 100 {
				|    loading
				|}
			";
			let data = Range { short_range: 25, long_range: 100, requires_ammunition: false, requires_loading: true };
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
