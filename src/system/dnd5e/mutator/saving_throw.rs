use crate::system::dnd5e::{character::DerivedBuilder, Ability};

#[derive(Clone)]
pub enum AddSavingThrow {
	Proficiency(Ability),
	Advantage(Ability, Option<String>),
}

impl super::Mutator for AddSavingThrow {
	fn scope_id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		match self {
			Self::Proficiency(ability) => {
				stats.add_saving_throw(*ability);
			}
			Self::Advantage(ability, target) => {
				stats.add_saving_throw_modifier(*ability, target.clone());
			}
		}
	}
}
