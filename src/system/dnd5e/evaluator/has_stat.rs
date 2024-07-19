use std::str::FromStr;

use crate::{
	kdl_ext::NodeContext,
	system::dnd5e::data::{character::Character, Ability},
	utility::NotInList,
};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};

#[derive(Clone, PartialEq, Debug)]
pub enum HasStat {
	AbilityScore { ability: Ability, score: u32 },
	//AbilityModifier,
	//SkillModifier,
	//Speed,
	//Sense,
	//Initiative,
	//ArmorClass,
}

crate::impl_trait_eq!(HasStat);
kdlize::impl_kdl_node!(HasStat, "stat");

impl crate::system::Evaluator for HasStat {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(match self {
			Self::AbilityScore { ability, score } => {
				format!("{} score is at least {score}", ability.to_string())
			}
		})
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		match self {
			Self::AbilityScore { ability, score } => {
				let ability_score = character.ability_scores().get(*ability).score();
				if *ability_score >= *score {
					return Ok(());
				}
				Err(format!("{} score is < {score}", ability.to_string()))
			}
		}
	}
}

impl FromKdl<NodeContext> for HasStat {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		match entry.type_req()? {
			"AbilityScore" => {
				let ability = Ability::from_str(entry.as_str_req()?)?;
				let score = node.next_i64_req()? as u32;
				Ok(Self::AbilityScore { ability, score })
			}
			type_id => Err(NotInList(type_id.into(), vec!["AbilityScore"]).into()),
		}
	}
}

impl AsKdl for HasStat {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::AbilityScore { ability, score } => {
				node.with_entry_typed(ability.to_string(), "AbilityScore").with_entry(*score as i64)
			}
		}
	}
}
