use crate::{
	kdl_ext::NodeQueryExt,
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode, Value},
	utility::{Dependencies, Evaluator, Mutator},
};

#[derive(Clone, Debug)]
pub struct AddMaxHitPoints {
	pub id: Option<String>,
	pub value: Value<i32>,
}

impl KDLNode for AddMaxHitPoints {
	fn id() -> &'static str {
		"add_max_hit_points"
	}
}

impl Mutator for AddMaxHitPoints {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn dependencies(&self) -> Dependencies {
		self.value.dependencies()
	}

	fn data_id(&self) -> Option<&str> {
		self.id.as_ref().map(String::as_str)
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		let value = self.value.evaluate(stats);
		stats.max_hit_points_mut().push(value, source);
	}
}

impl FromKDL<DnD5e> for AddMaxHitPoints {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut crate::kdl_ext::ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let id = node.get_str_opt("id")?.map(str::to_owned);
		// TODO: let value = Value::from_kdl(node, value_idx, system)?;
		let value = Value::default();
		Ok(Self { id, value })
	}
}

// TODO: Test AddMaxHitPoints
