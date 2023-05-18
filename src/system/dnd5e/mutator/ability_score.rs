use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt, ValueExt},
	system::dnd5e::data::{
		character::{AbilityScoreBonus, Character},
		description, Ability,
	},
	utility::{Mutator, Selector, SelectorMetaVec},
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct AbilityScoreChange {
	pub ability: Selector<Ability>,
	pub operations: Vec<AbilityScoreOp>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum AbilityScoreOp {
	Bonus {
		value: u32,
		max_total_score: Option<u32>,
	},
	IncreaseMax {
		value: u32,
	},
}

crate::impl_trait_eq!(AbilityScoreChange);
crate::impl_kdl_node!(AbilityScoreChange, "ability_score");

impl Mutator for AbilityScoreChange {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		let ability = match &self.ability {
			Selector::Specific(ability) => format!("Your {} score", ability.long_name()),
			Selector::Any { .. } => "One ability score of your choice".to_owned(),
			Selector::AnyOf { options, .. } => format!(
				"One ability score of {}",
				options
					.iter()
					.map(Ability::long_name)
					.collect::<Vec<_>>()
					.join(", ")
			),
		};
		let op_descs = self
			.operations
			.iter()
			.map(|op| match op {
				AbilityScoreOp::Bonus {
					value,
					max_total_score,
				} => {
					let bonus_txt = format!("increases by {value}");
					let max_txt = max_total_score
						.as_ref()
						.map(|value| format!(" if the total score is no more than {value}"));
					format!("{bonus_txt}{}", max_txt.unwrap_or_default())
				}
				AbilityScoreOp::IncreaseMax { value } => {
					format!("increases its maximum to at-least {value}")
				}
			})
			.collect::<Vec<_>>();

		description::Section {
			title: Some("Ability Score Increase".into()),
			content: format!("{ability}; {}.", op_descs.join(", ")),
			selectors: SelectorMetaVec::default().with_enum("Ability", &self.ability),
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.ability.set_data_path(parent);
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		if let Some(ability) = stats.resolve_selector(&self.ability) {
			for operation in &self.operations {
				match operation {
					AbilityScoreOp::Bonus {
						value,
						max_total_score,
					} => {
						stats.ability_scores_mut().push_bonus(
							ability,
							AbilityScoreBonus {
								value: *value,
								max_total: *max_total_score,
							},
							parent.to_owned(),
						);
					}
					AbilityScoreOp::IncreaseMax { value } => {
						stats.ability_scores_mut().increase_maximum(
							ability,
							*value,
							parent.to_owned(),
						);
					}
				}
			}
		}
	}
}

impl FromKDL for AbilityScoreChange {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let ability = {
			let mut ctx = ctx.next_node();
			let node = node.query_req("scope() > ability")?;
			let entry = node.entry_req(ctx.consume_idx())?;
			Selector::from_kdl(node, entry, &mut ctx, |kdl| {
				Ok(Ability::from_str(kdl.as_str_req()?)?)
			})?
		};
		let mut operations = Vec::new();
		for node in node.query_all("scope() > bonus")? {
			let mut ctx = ctx.next_node();
			let value = node.get_i64_req(ctx.consume_idx())? as u32;
			let max_total_score = node.get_i64_opt("max-total")?.map(|v| v as u32);
			operations.push(AbilityScoreOp::Bonus {
				value,
				max_total_score,
			});
		}
		for node in node.query_all("scope() > increase-max")? {
			let mut ctx = ctx.next_node();
			let value = node.get_i64_req(ctx.consume_idx())? as u32;
			operations.push(AbilityScoreOp::IncreaseMax { value });
		}
		Ok(Self {
			ability,
			operations,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::{core::NodeRegistry, dnd5e::BoxedMutator};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<AbilityScoreChange>(doc)
		}

		#[test]
		fn specific() -> anyhow::Result<()> {
			let doc = "mutator \"ability_score\" {
				ability \"Specific\" \"Dex\"
				bonus 2
			}";
			let expected = AbilityScoreChange {
				ability: Selector::Specific(Ability::Dexterity),
				operations: vec![AbilityScoreOp::Bonus {
					value: 2,
					max_total_score: None,
				}],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn any() -> anyhow::Result<()> {
			let doc = "mutator \"ability_score\" {
				ability \"Any\" id=\"skillA\"
				bonus 1
			}";
			let expected = AbilityScoreChange {
				ability: Selector::Any {
					id: Some("skillA").into(),
					cannot_match: Default::default(),
				},
				operations: vec![AbilityScoreOp::Bonus {
					value: 1,
					max_total_score: None,
				}],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn cannot_match() -> anyhow::Result<()> {
			let doc = "mutator \"ability_score\" {
				ability \"Any\" id=\"skillA\" {
					cannot-match \"skillB\"
				}
				bonus 3
			}";
			let expected = AbilityScoreChange {
				ability: Selector::Any {
					id: Some("skillA").into(),
					cannot_match: vec!["skillB".into()],
				},
				operations: vec![AbilityScoreOp::Bonus {
					value: 3,
					max_total_score: None,
				}],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn max_total_score() -> anyhow::Result<()> {
			let doc = "mutator \"ability_score\" {
				ability \"Specific\" \"Con\"
				bonus 3 max-total=12
			}";
			let expected = AbilityScoreChange {
				ability: Selector::Specific(Ability::Constitution),
				operations: vec![AbilityScoreOp::Bonus {
					value: 3,
					max_total_score: Some(12),
				}],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn increase_max() -> anyhow::Result<()> {
			let doc = "mutator \"ability_score\" {
				ability \"Specific\" \"Con\"
				increase-max 24
			}";
			let expected = AbilityScoreChange {
				ability: Selector::Specific(Ability::Constitution),
				operations: vec![AbilityScoreOp::IncreaseMax { value: 24 }],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn multiple_operations() -> anyhow::Result<()> {
			let doc = "mutator \"ability_score\" {
				ability \"Specific\" \"Strength\"
				bonus 4 max-total=17
				increase-max 24
			}";
			let expected = AbilityScoreChange {
				ability: Selector::Specific(Ability::Strength),
				operations: vec![
					AbilityScoreOp::Bonus {
						value: 4,
						max_total_score: Some(17),
					},
					AbilityScoreOp::IncreaseMax { value: 24 },
				],
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
				character::{AbilityScore, Character, Persistent},
				Ability, Feature,
			},
			utility::Selector,
		};

		fn character(
			base_scores: Vec<(Ability, u32)>,
			mutators: Vec<AbilityScoreChange>,
			selections: PathMap<String>,
		) -> Character {
			let mut persistent = Persistent::default();
			for (ability, score) in base_scores {
				persistent.ability_scores[ability] = score;
			}
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
				vec![(Ability::Strength, 10)],
				vec![AbilityScoreChange {
					ability: Selector::Specific(Ability::Strength),
					operations: vec![AbilityScoreOp::Bonus {
						value: 1,
						max_total_score: None,
					}],
				}],
				Default::default(),
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Strength),
				AbilityScore::default()
					.with_total(11)
					.with_bonus((10, None, "Base Score".into(), true))
					.with_bonus((1, None, "AddAbilityScore".into(), true))
			);
		}

		#[test]
		fn selected_ability() {
			let character = character(
				vec![(Ability::Dexterity, 10)],
				vec![AbilityScoreChange {
					ability: Selector::Any {
						id: Default::default(),
						cannot_match: Default::default(),
					},
					operations: vec![AbilityScoreOp::Bonus {
						value: 5,
						max_total_score: None,
					}],
				}],
				[("AddAbilityScore", "dexterity".into())].into(),
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Dexterity),
				AbilityScore::default()
					.with_total(15)
					.with_bonus((10, None, "Base Score".into(), true))
					.with_bonus((5, None, "AddAbilityScore".into(), true))
			);
		}

		#[test]
		fn specific_ability_with_base() {
			let character = Character::from(Persistent {
				ability_scores: enum_map::enum_map! {
					Ability::Strength => 10,
					Ability::Dexterity => 15,
					Ability::Constitution => 7,
					Ability::Intelligence => 11,
					Ability::Wisdom => 12,
					Ability::Charisma => 18,
				},
				feats: vec![Feature {
					name: "AddAbilityScore".into(),
					mutators: vec![AbilityScoreChange {
						ability: Selector::Specific(Ability::Intelligence),
						operations: vec![AbilityScoreOp::Bonus {
							value: 3,
							max_total_score: None,
						}],
					}
					.into()],
					..Default::default()
				}
				.into()],
				..Default::default()
			});
			assert_eq!(
				*character.ability_scores().get(Ability::Strength),
				AbilityScore::default().with_total(10).with_bonus((
					10,
					None,
					"Base Score".into(),
					true
				))
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Dexterity),
				AbilityScore::default().with_total(15).with_bonus((
					15,
					None,
					"Base Score".into(),
					true
				))
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Constitution),
				AbilityScore::default().with_total(7).with_bonus((
					7,
					None,
					"Base Score".into(),
					true
				))
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Intelligence),
				AbilityScore::default()
					.with_total(14)
					.with_bonus((11, None, "Base Score".into(), true))
					.with_bonus((3, None, "AddAbilityScore".into(), true))
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Wisdom),
				AbilityScore::default().with_total(12).with_bonus((
					12,
					None,
					"Base Score".into(),
					true
				))
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Charisma),
				AbilityScore::default().with_total(18).with_bonus((
					18,
					None,
					"Base Score".into(),
					true
				))
			);
		}

		#[test]
		fn max_total_score() {
			let character = character(
				vec![(Ability::Strength, 10)],
				vec![AbilityScoreChange {
					ability: Selector::Specific(Ability::Strength),
					operations: vec![AbilityScoreOp::Bonus {
						value: 5,
						max_total_score: Some(13),
					}],
				}],
				Default::default(),
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Strength),
				AbilityScore::default()
					.with_total(10)
					.with_bonus((10, None, "Base Score".into(), true))
					.with_bonus((5, Some(13), "AddAbilityScore".into(), false))
			);
		}

		#[test]
		fn increase_max() {
			let character = character(
				vec![(Ability::Strength, 18)],
				vec![AbilityScoreChange {
					ability: Selector::Specific(Ability::Strength),
					operations: vec![
						AbilityScoreOp::IncreaseMax { value: 24 },
						AbilityScoreOp::Bonus {
							value: 5,
							max_total_score: None,
						},
					],
				}],
				Default::default(),
			);
			assert_eq!(
				*character.ability_scores().get(Ability::Strength),
				AbilityScore::default()
					.with_total(23)
					.with_bonus((18, None, "Base Score".into(), true))
					.with_bonus((5, None, "AddAbilityScore".into(), true))
					.with_max_inc((24, "AddAbilityScore".into()))
			);
		}
	}
}
