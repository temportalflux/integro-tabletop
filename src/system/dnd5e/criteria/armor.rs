use crate::system::dnd5e::{
	character::Character,
	item::{armor, ItemKind},
};
use std::collections::HashSet;

/// Checks if the character has armor equipped.
#[derive(Clone, PartialEq, Default)]
pub struct HasArmorEquipped {
	/// By default, this criteria checks if a piece of armor is equipped.
	/// If this flag is true, the criteria checks if no armor is equipped (or no armor of a particular set of types).
	pub inverted: bool,
	/// The list of armor types to check. If empty, all armor is considered.
	pub kinds: HashSet<armor::Kind>,
}
impl HasArmorEquipped {
	fn kind_list(&self, joiner: &str) -> Option<String> {
		if self.kinds.is_empty() {
			return None;
		}
		let mut sorted_kinds = self.kinds.iter().collect::<Vec<_>>();
		sorted_kinds.sort();
		let mut kinds = sorted_kinds
			.into_iter()
			.map(|kind| format!("{kind:?}").to_lowercase())
			.collect::<Vec<_>>();
		Some(match kinds.len() {
			0 => unimplemented!(),
			1 => kinds.into_iter().next().unwrap(),
			2 => kinds.join(format!(" {joiner} ").as_str()),
			_ => {
				if let Some(last) = kinds.last_mut() {
					*last = format!("{joiner} {last}");
				}
				kinds.join(", ")
			}
		})
	}
}
impl super::Criteria for HasArmorEquipped {
	fn evaluate(&self, character: &Character) -> Result<(), String> {
		for item in character.inventory.items_without_ids() {
			if !item.is_equipable() || !item.is_equipped() {
				continue;
			}
			let ItemKind::Equipment(equipment) = &item.kind else { continue; };
			let Some(armor) = &equipment.armor else { continue; };
			if self.kinds.is_empty() || self.kinds.contains(&armor.kind) {
				return match self.inverted {
					false => Ok(()),
					true => Err(format!("\"{}\" is already equipped.", item.name)),
				};
			}
		}
		match self.inverted {
			false => Err(format!(
				"No {}armor equipped",
				match self.kind_list("or") {
					None => String::new(),
					Some(kind_list) => format!("{kind_list} "),
				}
			)),
			true => Ok(()),
		}
	}
}
