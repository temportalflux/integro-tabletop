use crate::system::dnd5e::{character::Character, Ability};

#[derive(Clone)]
pub enum AddSavingThrow {
	Proficiency(Ability),
	Advantage(Ability, Option<String>),
}

impl super::Mutator for AddSavingThrow {
	fn apply<'c>(&self, stats: &mut Character) {
		match self {
			Self::Proficiency(ability) => {
				let source = stats.source_path();
				stats.saving_throws_mut().add_proficiency(*ability, source);
			}
			Self::Advantage(ability, target) => {
				let source = stats.source_path();
				stats
					.saving_throws_mut()
					.add_modifier(*ability, target.clone(), source);
			}
		}
	}
}
