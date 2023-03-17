use std::str::FromStr;

use crate::{
	kdl_ext::{DocumentExt, NodeExt, ValueExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{character::Character, Ability},
			FromKDL,
		},
	},
	utility::{Mutator, Selector, SelectorMeta, SelectorMetaVec},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddAbilityScore {
	pub ability: Selector<Ability>,
	pub value: i32,
	// TODO: This is unimplemented, the bonus/value should not be applied if the result will be more than this score.
	pub max_total_score: Option<u32>,
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

	fn selector_meta(&self) -> Option<Vec<SelectorMeta>> {
		SelectorMetaVec::default()
			.with_enum("Ability", &self.ability)
			.to_vec()
	}
}

impl FromKDL for AddAbilityScore {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let ability = {
			let mut value_idx = ValueIdx::default();
			let node = node.query_req("scope() > ability")?;
			let entry_idx = value_idx.next();
			let entry = node.entry_req(entry_idx)?;
			Selector::from_kdl(node, entry, &mut value_idx, |kdl| {
				Ok(Ability::from_str(kdl.as_str_req()?)?)
			})?
		};
		let bonus = node.query_i64_req("scope() > bonus", 0)? as i32;
		let max_total_score = node
			.query_i64_opt("scope() > max_total_score", 0)?
			.map(|v| v as u32);
		Ok(Self {
			ability,
			value: bonus,
			max_total_score,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::dnd5e::BoxedMutator;

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<AddAbilityScore>(doc)
		}

		#[test]
		fn specific() -> anyhow::Result<()> {
			let doc = "mutator \"add_ability_score\" {
				ability \"Specific\" \"Dex\"
				bonus 2
			}";
			let expected = AddAbilityScore {
				ability: Selector::Specific(Ability::Dexterity),
				value: 2,
				max_total_score: None,
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn any() -> anyhow::Result<()> {
			let doc = "mutator \"add_ability_score\" {
				ability \"Any\" id=\"skillA\"
				bonus 1
			}";
			let expected = AddAbilityScore {
				ability: Selector::Any {
					id: Some("skillA").into(),
					cannot_match: Default::default(),
				},
				value: 1,
				max_total_score: None,
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn cannot_match() -> anyhow::Result<()> {
			let doc = "mutator \"add_ability_score\" {
				ability \"Any\" id=\"skillA\" {
					cannot-match \"skillB\"
				}
				bonus 3
			}";
			let expected = AddAbilityScore {
				ability: Selector::Any {
					id: Some("skillA").into(),
					cannot_match: vec!["skillB".into()],
				},
				value: 3,
				max_total_score: None,
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn max_total_score() -> anyhow::Result<()> {
			let doc = "mutator \"add_ability_score\" {
				ability \"Specific\" \"Con\"
				bonus 3
				max_total_score 12
			}";
			let expected = AddAbilityScore {
				ability: Selector::Specific(Ability::Constitution),
				value: 3,
				max_total_score: Some(12),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::{
			path_map::PathMap,
			system::dnd5e::data::{
				character::{Character, Persistent},
				Ability, Feature, Score,
			},
			utility::Selector,
		};

		fn character(mutators: Vec<AddAbilityScore>, selections: PathMap<String>) -> Character {
			let mut persistent = Persistent::default();
			persistent.feats.push(
				Feature {
					name: "AddAbilityScore".into(),
					mutators: mutators.into_iter().map(|m| m.into()).collect(),
					..Default::default()
				}
				.into(),
			);
			persistent.selected_values = selections;
			Character::from(persistent)
		}

		#[test]
		fn specific_ability() {
			let character = character(
				vec![AddAbilityScore {
					ability: Selector::Specific(Ability::Strength),
					value: 1,
					max_total_score: None,
				}],
				Default::default(),
			);
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
			let character = character(
				vec![AddAbilityScore {
					ability: Selector::Any {
						id: Default::default(),
						cannot_match: Default::default(),
					},
					value: 5,
					max_total_score: None,
				}],
				[("AddAbilityScore", "dexterity".into())].into(),
			);
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
						max_total_score: None,
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

		#[test]
		fn max_total_score() {
			let character = character(
				vec![AddAbilityScore {
					ability: Selector::Specific(Ability::Strength),
					value: 5,
					max_total_score: Some(13),
				}],
				Default::default(),
			);
			assert_eq!(
				character.ability_score(Ability::Strength),
				(
					Score(10),
					vec![("".into(), 10), ("AddAbilityScore".into(), 1)]
				)
			);
		}
	}
}
