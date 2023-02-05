use super::Selector;
use crate::system::dnd5e::{character::CompiledStats, Ability, Character};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
}

impl super::Modifier for AddAbilityScore {
	fn scope_id(&self) -> Option<&str> {
		self.ability.id()
	}

	fn apply(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		let ability = match &self.ability {
			Selector::Specific(ability) => Some(*ability),
			_ => match char.get_selection(stats, &scope) {
				Some(value) => Ability::from_str(&value).ok(),
				None => None,
			},
		};
		if let Some(ability) = ability {
			stats.ability_scores[ability] += self.value;
		}
	}
}
