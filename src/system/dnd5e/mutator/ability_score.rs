use super::Selector;
use crate::system::dnd5e::{character::DerivedBuilder, Ability};
use std::str::FromStr;

#[derive(Clone)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
}

impl super::Mutator for AddAbilityScore {
	fn scope_id(&self) -> Option<&str> {
		self.ability.id()
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		let ability = match &self.ability {
			Selector::Specific(ability) => Some(*ability),
			_ => match stats.get_selection() {
				Some(value) => Ability::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(ability) = ability {
			stats.add_to_ability_score(ability, self.value);
		}
	}
}
