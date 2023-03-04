use crate::{
	system::dnd5e::{data::character::Character, KDLNode},
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
