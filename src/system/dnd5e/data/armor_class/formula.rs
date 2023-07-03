use super::BoundedAbility;
use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{character::Character, Ability},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorClassFormula {
	pub base: u32,
	pub bonuses: Vec<BoundedAbility>,
}

impl Default for ArmorClassFormula {
	fn default() -> Self {
		Self {
			base: 10,
			bonuses: vec![BoundedAbility {
				ability: Ability::Dexterity,
				min: None,
				max: None,
			}],
		}
	}
}

impl From<u32> for ArmorClassFormula {
	fn from(base: u32) -> Self {
		Self {
			base,
			bonuses: Vec::new(),
		}
	}
}

impl ArmorClassFormula {
	pub fn evaluate(&self, state: &Character) -> i32 {
		let bonus: i32 = self
			.bonuses
			.iter()
			.map(|bounded| bounded.evaluate(state))
			.sum();
		(self.base as i32) + bonus
	}
}

impl FromKDL for ArmorClassFormula {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let base = node.get_i64_req("base")? as u32;
		let mut bonuses = Vec::new();
		for mut node in &mut node.query_all("scope() > bonus")? {
			let ability = Ability::from_str(node.next_str_req()?)?;
			let min = node.get_i64_opt("min")?.map(|v| v as i32);
			let max = node.get_i64_opt("max")?.map(|v| v as i32);
			bonuses.push(BoundedAbility { ability, min, max });
		}
		Ok(Self { base, bonuses })
	}
}

impl AsKdl for ArmorClassFormula {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(("base", self.base as i64));
		for bonus in &self.bonuses {
			node.push_child({
				let mut node =
					NodeBuilder::default().with_entry_typed(bonus.ability.long_name(), "Ability");
				if let Some(min) = &bonus.min {
					node.push_entry(("min", *min as i64));
				}
				if let Some(max) = &bonus.max {
					node.push_entry(("max", *max as i64));
				}
				node.build("bonus")
			});
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn default() {
		assert_eq!(
			ArmorClassFormula::default(),
			ArmorClassFormula {
				base: 10,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: None,
				}],
			}
		);
	}

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "formula";

		#[test]
		fn base_only() -> anyhow::Result<()> {
			let doc = "formula base=12";
			let data = ArmorClassFormula {
				base: 12,
				bonuses: vec![],
			};
			assert_eq_fromkdl!(ArmorClassFormula, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn one_bonus_unbounded() -> anyhow::Result<()> {
			let doc = "
			|formula base=12 {
			|    bonus (Ability)\"Dexterity\"
			|}
			";
			let data = ArmorClassFormula {
				base: 12,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: None,
				}],
			};
			assert_eq_fromkdl!(ArmorClassFormula, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn one_bonus_bounded() -> anyhow::Result<()> {
			let doc = "
				|formula base=15 {
				|    bonus (Ability)\"Dexterity\" max=2
				|}
			";
			let data = ArmorClassFormula {
				base: 15,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: Some(2),
				}],
			};
			assert_eq_fromkdl!(ArmorClassFormula, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn multiple_bonus() -> anyhow::Result<()> {
			let doc = "
				|formula base=10 {
				|    bonus (Ability)\"Dexterity\"
				|    bonus (Ability)\"Wisdom\"
				|}
			";
			let data = ArmorClassFormula {
				base: 10,
				bonuses: vec![
					BoundedAbility {
						ability: Ability::Dexterity,
						min: None,
						max: None,
					},
					BoundedAbility {
						ability: Ability::Wisdom,
						min: None,
						max: None,
					},
				],
			};
			assert_eq_fromkdl!(ArmorClassFormula, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::system::dnd5e::data::character::Persistent;

		fn character(scores: &[(Ability, u32)]) -> Character {
			let mut persistent = Persistent::default();
			for (ability, score) in scores {
				persistent.ability_scores[*ability] = *score;
			}
			Character::from(persistent)
		}

		#[test]
		fn no_bonuses() {
			let formula = ArmorClassFormula {
				base: 10,
				bonuses: vec![],
			};
			let character = character(&[(Ability::Dexterity, 20)]);
			assert_eq!(formula.evaluate(&character), 10);
		}

		#[test]
		fn one_bonus() {
			let formula = ArmorClassFormula {
				base: 10,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: None,
				}],
			};
			let character = character(&[(Ability::Dexterity, 8)]);
			assert_eq!(formula.evaluate(&character), 9);
		}

		#[test]
		fn multiple_bonus() {
			let formula = ArmorClassFormula {
				base: 10,
				bonuses: vec![
					BoundedAbility {
						ability: Ability::Dexterity,
						min: None,
						max: None,
					},
					BoundedAbility {
						ability: Ability::Constitution,
						min: None,
						max: None,
					},
				],
			};
			let character = character(&[(Ability::Dexterity, 14), (Ability::Constitution, 12)]);
			assert_eq!(formula.evaluate(&character), 13);
		}

		#[test]
		fn ability_max() {
			let formula = ArmorClassFormula {
				base: 15,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: None,
					max: Some(2),
				}],
			};
			let character = character(&[(Ability::Dexterity, 18)]);
			assert_eq!(formula.evaluate(&character), 17);
		}

		#[test]
		fn ability_min() {
			let formula = ArmorClassFormula {
				base: 10,
				bonuses: vec![BoundedAbility {
					ability: Ability::Dexterity,
					min: Some(3),
					max: None,
				}],
			};
			let character = character(&[(Ability::Dexterity, 10)]);
			assert_eq!(formula.evaluate(&character), 13);
		}
	}
}
