use crate::system::dnd5e::character::{ArmorClassFormula, Character};

#[derive(Clone, PartialEq)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);
impl super::Mutator for AddArmorClassFormula {
	fn apply<'c>(&self, stats: &mut Character) {
		stats.armor_class_mut().push_formula(self.0.clone());
	}
}
