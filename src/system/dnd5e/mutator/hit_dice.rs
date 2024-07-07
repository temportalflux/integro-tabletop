use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::data::{character::Character, description, roll::EvaluatedRoll},
		mutator::ReferencePath,
		Mutator,
	},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, Debug, PartialEq)]
pub struct HitDice {
	roll: EvaluatedRoll,
}

crate::impl_trait_eq!(HitDice);
kdlize::impl_kdl_node!(HitDice, "hit_dice");

impl Mutator for HitDice {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		let content = match state {
			Some(state) => {
				let roll = self.roll.evaluate(state);
				format!("You gain {roll} hit dice")
			}
			None => {
				let amount = self.roll.amount.description();
				let die = self.roll.die.as_ref().map(|eval| eval.description()).flatten();
				if let Some((amount, die)) = amount.zip(die) {
					format!("You gain hit dice equivalent to {amount} {die}")
				} else {
					format!("You gain some hit dice")
				}
			}
		};

		description::Section { content: content.into(), ..Default::default() }
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		let roll = self.roll.evaluate(stats);
		stats.hit_dice_mut().push(roll, parent.display.clone());
	}
}

impl FromKdl<NodeContext> for HitDice {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let roll = EvaluatedRoll::from_kdl(node)?;
		Ok(Self { roll })
	}
}

impl AsKdl for HitDice {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node += self.roll.as_kdl();
		node
	}
}
