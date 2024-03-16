use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::{
		data::{
			character::{ActionBudgetKind, Character},
			description,
		},
		Value,
	},
	system::Mutator,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct AddToActionBudget {
	pub action_kind: ActionBudgetKind,
	pub amount: Value<i32>,
}

crate::impl_trait_eq!(AddToActionBudget);
kdlize::impl_kdl_node!(AddToActionBudget, "add_to_action_budget");

impl Mutator for AddToActionBudget {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			title: Some("Add to Action Budget".into()),
			content: format!(
				"You get {} additional {} on your turn{}.",
				match &self.amount {
					Value::Fixed(value) => value.to_string(),
					// TODO: Show a table for the Basis::Level, where the first column is
					// the class or character level, and the second column the (optional) value.
					Value::Evaluated(_basis) => "some".into(),
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
			)
			.into(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &ReferencePath) {
		let amount = self.amount.evaluate(stats);
		if amount >= 0 {
			stats
				.features_mut()
				.action_budget
				.push(self.action_kind, amount as u32, parent.display.clone());
		}
	}
}

impl FromKdl<NodeContext> for AddToActionBudget {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let action_kind = node.next_str_req_t::<ActionBudgetKind>()?;
		let amount = Value::from_kdl(node)?;
		Ok(Self { action_kind, amount })
	}
}

impl AsKdl for AddToActionBudget {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(self.action_kind.to_string());
		node += self.amount.as_kdl();
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(AddToActionBudget);

		#[test]
		fn action() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Action\" 1";
			let data = AddToActionBudget {
				action_kind: ActionBudgetKind::Action,
				amount: Value::Fixed(1),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn attack() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Attack\" 1";
			let data = AddToActionBudget {
				action_kind: ActionBudgetKind::Attack,
				amount: Value::Fixed(1),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn bonus() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Bonus\" 1";
			let data = AddToActionBudget {
				action_kind: ActionBudgetKind::Bonus,
				amount: Value::Fixed(1),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn reaction() -> anyhow::Result<()> {
			let doc = "mutator \"add_to_action_budget\" \"Reaction\" 1";
			let data = AddToActionBudget {
				action_kind: ActionBudgetKind::Reaction,
				amount: Value::Fixed(1),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{character::Persistent, Bundle};

		fn character(mutator: AddToActionBudget) -> Character {
			let mut persistent = Persistent::default();
			persistent.bundles.push(
				Bundle {
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
				amount: Value::Fixed(1),
			});
			let budget = &character.features().action_budget;
			assert_eq!(budget.get(ActionBudgetKind::Action), (2, &vec![(1, "Test".into())]));
		}

		#[test]
		fn attack() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Attack,
				amount: Value::Fixed(1),
			});
			let budget = &character.features().action_budget;
			assert_eq!(budget.get(ActionBudgetKind::Attack), (2, &vec![(1, "Test".into())]));
		}

		#[test]
		fn bonus() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Bonus,
				amount: Value::Fixed(1),
			});
			let budget = &character.features().action_budget;
			assert_eq!(budget.get(ActionBudgetKind::Bonus), (2, &vec![(1, "Test".into())]));
		}

		#[test]
		fn reaction() {
			let character = character(AddToActionBudget {
				action_kind: ActionBudgetKind::Reaction,
				amount: Value::Fixed(1),
			});
			let budget = &character.features().action_budget;
			assert_eq!(budget.get(ActionBudgetKind::Reaction), (2, &vec![(1, "Test".into())]));
		}
	}
}
