use super::Selector;
use crate::system::dnd5e::{character::DerivedBuilder, Ability};

#[derive(Clone)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
}

impl super::Mutator for AddAbilityScore {
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		if let Some(ability) = stats.resolve_selector(&self.ability) {
			stats.add_to_ability_score(ability, self.value);
		}
	}
}
