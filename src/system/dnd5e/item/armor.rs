use crate::system::dnd5e::{
	character::{ArmorClassFormula, DerivedBuilder},
	mutator,
};

#[derive(Clone, PartialEq)]
pub struct Armor {
	pub kind: Kind,
	pub formula: ArmorClassFormula,
	/// The minimum expected strength score to use this armor.
	/// If provided, characters with a value less than this are hindered (reduced speed).
	/// TODO: Reduce speed by 10 if strength score not met
	pub min_strength_score: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
	Light,
	Medium,
	Heavy,
}
impl ToString for Kind {
	fn to_string(&self) -> String {
		format!("{self:?}")
	}
}

impl mutator::Container for Armor {
	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		stats.armor_class_mut().push(self.formula.clone());
	}
}
