// Components:
// Verbal
// Somatic
// Material (string + consumes=bool)
// can have multiple material component entries, which are collected into a vec

use crate::{
	kdl_ext::{AsKdl, NodeBuilder, NodeContext, NodeExt},
	utility::NotInList,
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Components {
	pub verbal: bool,
	pub somatic: bool,
	pub materials: Vec<(String, /*consumes on cast*/ bool)>,
}

impl Components {
	/// Queries the children of `parent` for any nodes named `component`,
	/// and extends the default `Components` with all identified children.
	pub fn from_kdl_all(parent: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let mut components = Self::default();
		for node in parent.query_all("scope() > component")? {
			let mut ctx = ctx.next_node();
			match node.get_str_req(ctx.consume_idx())? {
				"Verbal" => components.verbal = true,
				"Somatic" => components.somatic = true,
				"Material" => {
					let material = node.get_str_req(ctx.consume_idx())?.to_owned();
					let consume_on_cast = node.get_bool_opt("consumes")?.unwrap_or_default();
					components.materials.push((material, consume_on_cast));
				}
				name => {
					return Err(
						NotInList(name.into(), vec!["Verbal", "Somatic", "Material"]).into(),
					)
				}
			}
		}
		Ok(components)
	}
}
// TODO AsKdl: from/as tests for spell components
impl AsKdl for Components {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if self.verbal {
			node.push_child_entry("component", "Verbal");
		}
		if self.somatic {
			node.push_child_entry("component", "Somatic");
		}
		for (material, consumed) in &self.materials {
			node.push_child({
				let mut node = NodeBuilder::default()
					.with_entry("Material")
					.with_entry(material.clone());
				if *consumed {
					node.push_entry(("consumes", true));
				}
				node.build("component")
			});
		}
		node
	}
}
