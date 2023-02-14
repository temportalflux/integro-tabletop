use crate::system::dnd5e::character::DerivedBuilder;

#[derive(Clone)]
pub struct AddMaxSpeed(pub String, pub i32);

impl super::Modifier for AddMaxSpeed {
	fn scope_id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		stats.add_max_speed(self.0.clone(), self.1);
	}
}

#[derive(Clone)]
pub struct AddMaxSense(pub String, pub i32);

impl super::Modifier for AddMaxSense {
	fn scope_id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		stats.add_max_sense(self.0.clone(), self.1);
	}
}
