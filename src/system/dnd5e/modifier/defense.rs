use crate::system::dnd5e::character::DerivedBuilder;
use enum_map::Enum;

#[derive(Clone, Copy, PartialEq, Enum, Debug)]
pub enum Defense {
	Resistant,
	Immune,
	Vulnerable,
}

#[derive(Clone)]
pub struct AddDefense(pub Defense, pub String);

impl super::Modifier for AddDefense {
	fn scope_id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		stats.add_defense(self.0, self.1.clone());
	}
}
