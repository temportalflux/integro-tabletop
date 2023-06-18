use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	utility::NotInList,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AreaOfEffect {
	Cone { length: u32 },
	Cube { size: u32 },
	Cylinder { radius: u32, height: u32 },
	Line { width: u32, length: u32 },
	Sphere { radius: u32 },
}

impl FromKDL for AreaOfEffect {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Cone" => Ok(Self::Cone {
				length: node.get_i64_req("length")? as u32,
			}),
			"Cube" => Ok(Self::Cube {
				size: node.get_i64_req("size")? as u32,
			}),
			"Cylinder" => Ok(Self::Cylinder {
				radius: node.get_i64_req("radius")? as u32,
				height: node.get_i64_req("height")? as u32,
			}),
			"Line" => Ok(Self::Line {
				width: node.get_i64_req("width")? as u32,
				length: node.get_i64_req("length")? as u32,
			}),
			"Sphere" => Ok(Self::Sphere {
				radius: node.get_i64_req("radius")? as u32,
			}),
			name => Err(NotInList(
				name.into(),
				vec!["Cone", "Cube", "Cylinder", "Line", "Sphere"],
			)
			.into()),
		}
	}
}
impl AsKdl for AreaOfEffect {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Cone { length } => node
				.with_entry("Cone")
				.with_entry(("length", *length as i64)),
			Self::Cube { size } => node.with_entry("Cube").with_entry(("size", *size as i64)),
			Self::Cylinder { radius, height } => node
				.with_entry("Cylinder")
				.with_entry(("radius", *radius as i64))
				.with_entry(("height", *height as i64)),
			Self::Line { width, length } => node
				.with_entry("Line")
				.with_entry(("width", *width as i64))
				.with_entry(("length", *length as i64)),
			Self::Sphere { radius } => node
				.with_entry("Sphere")
				.with_entry(("radius", *radius as i64)),
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "area_of_effect";

		#[test]
		fn cone() -> anyhow::Result<()> {
			let doc = "area_of_effect \"Cone\" length=30";
			let data = AreaOfEffect::Cone { length: 30 };
			assert_eq_fromkdl!(AreaOfEffect, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn cube() -> anyhow::Result<()> {
			let doc = "area_of_effect \"Cube\" size=10";
			let data = AreaOfEffect::Cube { size: 10 };
			assert_eq_fromkdl!(AreaOfEffect, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn cylinder() -> anyhow::Result<()> {
			let doc = "area_of_effect \"Cylinder\" radius=10 height=40";
			let data = AreaOfEffect::Cylinder {
				radius: 10,
				height: 40,
			};
			assert_eq_fromkdl!(AreaOfEffect, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn line() -> anyhow::Result<()> {
			let doc = "area_of_effect \"Line\" width=5 length=60";
			let data = AreaOfEffect::Line {
				width: 5,
				length: 60,
			};
			assert_eq_fromkdl!(AreaOfEffect, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn sphere() -> anyhow::Result<()> {
			let doc = "area_of_effect \"Sphere\" radius=20";
			let data = AreaOfEffect::Sphere { radius: 20 };
			assert_eq_fromkdl!(AreaOfEffect, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
