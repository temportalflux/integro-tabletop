use std::str::FromStr;
use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::{
		character::{ActionBudgetKind, Character},
		scaling,
	},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddToActionBudget {
	pub action_kind: ActionBudgetKind,
	pub amount: scaling::Value<u32>,
}

crate::impl_trait_eq!(AddToActionBudget);
crate::impl_kdl_node!(AddToActionBudget, "add_to_action_budget");

impl Mutator for AddToActionBudget {
	type Target = Character;

	fn name(&self) -> Option<String> {
		Some("Add to Action Budget".into())
	}

	fn description(&self) -> Option<String> {
		Some(format!(
			"You get {} additional {} on your turn{}.",
			match &self.amount {
				scaling::Value::Fixed(value) => value.to_string(),
				// TODO: Show a table for the Basis::Level, where the first column is
				// the class or character level, and the second column the (optional) value.
				scaling::Value::Scaled(_basis) => "some".into(),
			},
			match &self.action_kind {
				ActionBudgetKind::Attack => "attack(s)",
				ActionBudgetKind::Action => "action(s)",
				ActionBudgetKind::Bonus => "bonus action(s)",
				ActionBudgetKind::Reaction => "reaction(s)",
			},
			match &self.action_kind {
				ActionBudgetKind::Attack => " when you use the attack action",
				_ => "",
			}
		))
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		if let Some(amount) = self.amount.evaluate(stats) {
			stats
				.actions_mut()
				.action_budget
				.push(self.action_kind, amount, parent.into());
		}
	}
}

impl FromKDL for AddToActionBudget {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let action_kind = ActionBudgetKind::from_str(node.get_str_req(ctx.consume_idx())?)?;
		let amount = scaling::Value::from_kdl(node, ctx)?;
		Ok(Self {
			action_kind,
			amount,
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
			NodeRegistry::defaultmut_parse_kdl::<AddToActionBudget>(doc)
		}

		#[test]
		fn action() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Action\" 1";
			let expected = AddToActionBudget {
				action_kind: ActionBudgetKind::Action,
				amount: scaling::Value::Fixed(1),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn attack() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Attack\" 1";
			let expected = AddToActionBudget {
				action_kind: ActionBudgetKind::Attack,
				amount: scaling::Value::Fixed(1),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn bonus() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Bonus\" 1";
			let expected = AddToActionBudget {
				action_kind: ActionBudgetKind::Bonus,
				amount: scaling::Value::Fixed(1),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn reaction() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Reaction\" 1";
			let expected = AddToActionBudget {
				action_kind: ActionBudgetKind::Reaction,
				amount: scaling::Value::Fixed(1),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{character::Persistent, Feature};

		fn character(mutator: AddToActionBudget) -> Character {
			let mut persistent = Persistent::default();
			persistent.feats.push(
				Feature {
					name: "Test".into(),
					mutators: vec![mutator.into()],
					..Default::default()
				}
				.into(),
			);
			Character::from(persistent)
		}

		#[test]
		fn action() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Action,
				amount: scaling::Value::Fixed(1),
			});
			let budget = &character.actions().action_budget;
			assert_eq!(
				budget.get(ActionBudgetKind::Action),
				(2, &vec![(1, "Test".into())])
			);
		}

		#[test]
		fn attack() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Attack,
				amount: scaling::Value::Fixed(1),
			});
			let budget = &character.actions().action_budget;
			assert_eq!(
				budget.get(ActionBudgetKind::Attack),
				(2, &vec![(1, "Test".into())])
			);
		}

		#[test]
		fn bonus() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Bonus,
				amount: scaling::Value::Fixed(1),
			});
			let budget = &character.actions().action_budget;
			assert_eq!(
				budget.get(ActionBudgetKind::Bonus),
				(2, &vec![(1, "Test".into())])
			);
		}

		#[test]
		fn reaction() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Reaction,
				amount: scaling::Value::Fixed(1),
			});
			let budget = &character.actions().action_budget;
			assert_eq!(
				budget.get(ActionBudgetKind::Reaction),
				(2, &vec![(1, "Test".into())])
			);
		}
	}
}
