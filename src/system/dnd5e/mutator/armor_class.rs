use crate::system::dnd5e::character::{ArmorClassFormula, DerivedBuilder};

#[derive(Clone, PartialEq)]
pub struct AddArmorClassFormula(pub ArmorClassFormula);
impl super::Mutator for AddArmorClassFormula {
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		stats.armor_class_mut().push(self.0.clone());
	}
}
