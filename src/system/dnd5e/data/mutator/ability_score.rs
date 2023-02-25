use crate::{
	system::dnd5e::data::{character::Character, Ability},
	utility::{Mutator, Selector},
};

#[derive(Clone)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
}

impl Mutator for AddAbilityScore {
	type Target = Character;

	fn node_id(&self) -> &'static str {
		"add_ability_score"
	}

	fn apply<'c>(&self, stats: &mut Character) {
		if let Some(ability) = stats.resolve_selector(&self.ability) {
			let source = stats.source_path();
			stats
				.ability_scores_mut()
				.push_bonus(ability, self.value, source);
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
				Score(1),
				vec![("".into(), 0), ("AddAbilityScore".into(), 1)]
			)
		);
	}

	#[test]
	fn selected_ability() {
		let character = Character::from(Persistent {
			feats: vec![Feature {
				name: "AddAbilityScore".into(),
				mutators: vec![AddAbilityScore {
					ability: Selector::Any { id: None },
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
				Score(5),
				vec![("".into(), 0), ("AddAbilityScore".into(), 5)]
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
