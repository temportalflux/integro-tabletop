use crate::{
	system::dnd5e::{data::character::Character, KDLNode, Value},
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
