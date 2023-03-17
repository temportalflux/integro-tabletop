use crate::{
	kdl_ext::{NodeExt, ValueIdx},
	system::{core::NodeRegistry, dnd5e::FromKDL},
	GeneralError,
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
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		match node.get_str_req(value_idx.next())? {
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
			name => Err(GeneralError(format!(
				"Invalid area of effect {name:?}, \
				expected Cone, Cube, Cylinder, Line, or Sphere"
			))
			.into()),
		}
	}
}
