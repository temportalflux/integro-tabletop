use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	utility::NotInList,
};

// Components:
// Verbal
// Somatic
// Material (string + consumes=bool)
// can have multiple material component entries, which are collected into a vec
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Components {
	pub verbal: bool,
	pub somatic: bool,
	pub materials: Vec<(String, /*consumes on cast*/ bool)>,
}

impl FromKDL for Components {
	/// Queries the children of `parent` for any nodes named `component`,
	/// and extends the default `Components` with all identified children.
	fn from_kdl<'doc>(parent: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut components = Self::default();
		for mut node in parent.query_all("scope() > component")? {
			match node.next_str_req()? {
				"Verbal" => components.verbal = true,
				"Somatic" => components.somatic = true,
				"Material" => {
					let material = node.next_str_req()?.to_owned();
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

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "list";

		#[test]
		fn verbal() -> anyhow::Result<()> {
			let doc = "
				|list {
				|    component \"Verbal\"
				|}
			";
			let data = Components {
				verbal: true,
				somatic: false,
				materials: vec![],
			};
			assert_eq_fromkdl!(Components, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn somatic() -> anyhow::Result<()> {
			let doc = "
				|list {
				|    component \"Somatic\"
				|}
			";
			let data = Components {
				verbal: false,
				somatic: true,
				materials: vec![],
			};
			assert_eq_fromkdl!(Components, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn material() -> anyhow::Result<()> {
			let doc = "
				|list {
				|    component \"Material\" \"a swatch of wool\"
				|}
			";
			let data = Components {
				verbal: false,
				somatic: false,
				materials: vec![("a swatch of wool".into(), false)],
			};
			assert_eq_fromkdl!(Components, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn material_consumed() -> anyhow::Result<()> {
			let doc = "
				|list {
				|    component \"Material\" \"a swatch of wool\" consumes=true
				|}
			";
			let data = Components {
				verbal: false,
				somatic: false,
				materials: vec![("a swatch of wool".into(), true)],
			};
			assert_eq_fromkdl!(Components, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
