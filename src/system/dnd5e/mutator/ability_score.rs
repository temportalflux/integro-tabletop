use super::Selector;
use crate::system::dnd5e::{character::Character, Ability};

#[derive(Clone)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
}

impl super::Mutator for AddAbilityScore {
	fn apply<'c>(&self, stats: &mut Character) {
		if let Some(ability) = stats.resolve_selector(&self.ability) {
			let source = stats.source_path();
			stats
				.ability_scores_mut()
				.push_bonus(ability, self.value, source);
		}
	}
}
