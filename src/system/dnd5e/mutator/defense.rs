use crate::system::dnd5e::character::Character;
use enum_map::Enum;

#[derive(Clone, Copy, PartialEq, Enum, Debug)]
pub enum Defense {
	Resistant,
	Immune,
	Vulnerable,
}

#[derive(Clone)]
pub struct AddDefense(pub Defense, pub String);
impl super::Mutator for AddDefense {
	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats.defenses_mut().push(self.0, self.1.clone(), source);
	}
}
