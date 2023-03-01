use crate::{
	system::dnd5e::{data::character::Character, Value},
	utility::{Evaluator, Mutator},
};

#[derive(Clone)]
pub struct AddMaxHitPoints {
	pub id: Option<String>,
	pub value: Value<i32>,
}

impl Mutator for AddMaxHitPoints {
	type Target = Character;

	fn node_id(&self) -> &'static str {
		"add_max_hit_points"
	}

	fn dependencies(&self) -> Option<Vec<&'static str>> {
		// TODO: This should actually be derived from what the dependencies are of the provided evaluator
		Some(vec!["add_ability_score"])
	}

	fn id(&self) -> Option<&str> {
		self.id.as_ref().map(String::as_str)
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		let value = self.value.evaluate(stats);
		stats.max_hit_points_mut().push(value, source);
	}
}
