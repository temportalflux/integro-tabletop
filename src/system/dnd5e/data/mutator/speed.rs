use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Mutator,
};

#[derive(Clone, Debug)]
pub struct AddMaxSpeed(pub String, pub i32);

impl KDLNode for AddMaxSpeed {
	fn id() -> &'static str {
		"add_max_speed"
	}
}

impl Mutator for AddMaxSpeed {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.speeds_mut().push_max(self.0.clone(), self.1, source);
	}
}

impl FromKDL<DnD5e> for AddMaxSpeed {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let kind = node.get_str(value_idx.next())?.to_owned();
		let amount = node.get_i64(value_idx.next())? as i32;
		Ok(Self(kind, amount))
	}
}

#[derive(Clone, Debug)]
pub struct AddMaxSense(pub String, pub i32);

impl KDLNode for AddMaxSense {
	fn id() -> &'static str {
		"add_max_sense"
	}
}

impl Mutator for AddMaxSense {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.senses_mut().push_max(self.0.clone(), self.1, source);
	}
}

impl FromKDL<DnD5e> for AddMaxSense {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let kind = node.get_str(value_idx.next())?.to_owned();
		let amount = node.get_i64(value_idx.next())? as i32;
		Ok(Self(kind, amount))
	}
}

// TODO: Test AddMaxSpeed
// TODO: Test AddMaxSense
