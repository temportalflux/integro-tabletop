use crate::{
	system::dnd5e::data::{character::Character, Ability},
	utility::{Mutator, Selector},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
}

crate::impl_trait_eq!(AddAbilityScore);
crate::impl_kdl_node!(AddAbilityScore, "add_ability_score");

impl Mutator for AddAbilityScore {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		self.ability.set_data_path(parent);
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		if let Some(ability) = stats.resolve_selector(&self.ability) {
			stats
				.ability_scores_mut()
				.push_bonus(ability, self.value, parent.to_owned());
		}
	}
}

#[cfg(test)]
mod test {
	use super::AddAbilityScore;
	use crate::{
		path_map::PathMap,
		system::dnd5e::data::{
			character::{Character, Persistent},
			Ability, Feature, Score,
		},
		utility::Selector,
	};

	#[test]
	fn specific_ability() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddAbilityScore".into(),
				mutators: vec![AddAbilityScore {
					ability: Selector::Specific(Ability::Strength),
					value: 1,
				}
				.into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.ability_score(Ability::Strength),
			(
				Score(11),
				vec![("".into(), 10), ("AddAbilityScore".into(), 1)]
			)
		);
	}

	#[test]
	fn selected_ability() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddAbilityScore".into(),
				mutators: vec![AddAbilityScore {
					ability: Selector::Any {
						id: Default::default(),
					},
					value: 5,
				}
				.into()],
				..Default::default()
			}
			.into()],
			selected_values: PathMap::from([("AddAbilityScore", "dexterity".into())]),
			..Default::default()
		});
		assert_eq!(
			character.ability_score(Ability::Dexterity),
			(
				Score(15),
				vec![("".into(), 10), ("AddAbilityScore".into(), 5)]
			)
		);
	}

	#[test]
	fn specific_ability_with_base() {
		let character = Character::from(Persistent {
			ability_scores: enum_map::enum_map! {
				Ability::Strength => Score(10),
				Ability::Dexterity => Score(15),
				Ability::Constitution => Score(7),
				Ability::Intelligence => Score(11),
				Ability::Wisdom => Score(12),
				Ability::Charisma => Score(18),
			},
			feats: vec![Feature {
				name: "AddAbilityScore".into(),
				mutators: vec![AddAbilityScore {
					ability: Selector::Specific(Ability::Intelligence),
					value: 3,
				}
				.into()],
				..Default::default()
			}
			.into()],
			..Default::default()
		});
		assert_eq!(
			character.ability_score(Ability::Strength),
			(Score(10), vec![("".into(), 10)])
		);
		assert_eq!(
			character.ability_score(Ability::Dexterity),
			(Score(15), vec![("".into(), 15)])
		);
		assert_eq!(
			character.ability_score(Ability::Constitution),
			(Score(7), vec![("".into(), 7)])
		);
		assert_eq!(
			character.ability_score(Ability::Intelligence),
			(
				Score(14),
				vec![("".into(), 11), ("AddAbilityScore".into(), 3)]
			)
		);
		assert_eq!(
			character.ability_score(Ability::Wisdom),
			(Score(12), vec![("".into(), 12)])
		);
		assert_eq!(
			character.ability_score(Ability::Charisma),
			(Score(18), vec![("".into(), 18)])
		);
	}
}
