use crate::system::dnd5e::character::Character;

#[derive(Clone)]
pub struct AddMaxSpeed(pub String, pub i32);

impl super::Mutator for AddMaxSpeed {
	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.speeds_mut().push_max(self.0.clone(), self.1, source);
	}
}

#[derive(Clone)]
pub struct AddMaxSense(pub String, pub i32);

impl super::Mutator for AddMaxSense {
	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.senses_mut().push_max(self.0.clone(), self.1, source);
	}
}
