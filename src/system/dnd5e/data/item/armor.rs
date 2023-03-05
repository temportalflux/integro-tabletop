use std::str::FromStr;

use enumset::EnumSetType;

use crate::{
	system::dnd5e::data::{character::Character, ArmorClassFormula},
	utility::MutatorGroup,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Armor {
	pub kind: Kind,
	pub formula: ArmorClassFormula,
	/// The minimum expected strength score to use this armor.
	/// If provided, characters with a value less than this are hindered (reduced speed).
	/// TODO: Reduce speed by 10 if strength score not met
	pub min_strength_score: Option<u32>,
}

#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
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
impl FromStr for Kind {
	type Err = crate::GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"light" => Ok(Self::Light),
			"medium" => Ok(Self::Medium),
			"heavy" => Ok(Self::Heavy),
			_ => Err(crate::GeneralError(format!(
				"{s:?} is not a valid armor kind: {:?}.",
				enumset::EnumSet::<Kind>::all()
					.into_iter()
					.collect::<Vec<_>>(),
			))),
		}
	}
}

impl MutatorGroup for Armor {
	type Target = Character;

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		stats
			.armor_class_mut()
			.push_formula(self.formula.clone(), source);
	}
}
