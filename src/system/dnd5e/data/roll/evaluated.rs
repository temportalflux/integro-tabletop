use crate::kdl_ext::NodeContext;
use crate::{
	system::dnd5e::{
		data::character::Character,
		data::roll::{Die, Roll},
		Value,
	},
	utility::Dependencies,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EvaluatedRoll {
	pub amount: Value<i32>,
	pub die: Option<Value<i32>>,
}

impl<T> From<T> for EvaluatedRoll
where
	Roll: From<T>,
{
	fn from(value: T) -> Self {
		let roll = Roll::from(value);
		Self {
			amount: Value::Fixed(roll.amount as i32),
			die: roll.die.map(|die| Value::Fixed(die.value() as i32)),
		}
	}
}

impl EvaluatedRoll {
	pub fn dependencies(&self) -> Dependencies {
		let mut deps = self.amount.dependencies();
		if let Some(die_value) = &self.die {
			deps = deps.join(die_value.dependencies());
		}
		deps
	}

	pub fn evaluate(&self, character: &Character) -> Roll {
		let amount = self.amount.evaluate(character) as u32;
		let die = match &self.die {
			None => None,
			Some(value) => {
				let die_value = value.evaluate(character) as u32;
				Die::try_from(die_value).ok()
			}
		};
		Roll { amount, die }
	}
}

impl FromKdl<NodeContext> for EvaluatedRoll {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		if let Some(entry) = node.next_opt() {
			return Ok(Self::from(Roll::from_kdl_value(entry.value())?));
		}
		let amount = node.query_req_t::<Value<i32>>("scope() > amount")?;
		let die = node.query_opt_t::<Value<i32>>("scope() > die")?;
		Ok(Self { amount, die })
	}
}

impl AsKdl for EvaluatedRoll {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			// These first two are when the EvaluatedRoll is a fixed Roll, and thus can be serialized as such
			Self {
				amount: Value::Fixed(amt),
				die: None,
			} => node.with_entry(*amt as i64),
			Self {
				amount: Value::Fixed(amt),
				die: Some(Value::Fixed(die)),
			} => node.with_entry_typed(format!("{amt}d{die}"), "Roll"),
			// While this one puts the amount and die into child nodes for evaluator serialization
			Self { amount, die } => {
				node.push_child_t("amount", amount);
				if let Some(die) = die {
					node.push_child_t("die", die);
				}
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::{core::NodeRegistry, dnd5e::evaluator::GetProficiencyBonus},
		};

		static NODE_NAME: &str = "roll";

		fn node_ctx() -> NodeContext {
			NodeContext::registry(NodeRegistry::default_with_eval::<GetProficiencyBonus>())
		}

		#[test]
		fn basic_fixed() -> anyhow::Result<()> {
			let doc = "roll 1";
			let data = EvaluatedRoll {
				amount: Value::Fixed(1),
				die: None,
			};
			assert_eq_fromkdl!(EvaluatedRoll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn basic_die() -> anyhow::Result<()> {
			let doc = "roll (Roll)\"3d4\"";
			let data = EvaluatedRoll {
				amount: Value::Fixed(3),
				die: Some(Value::Fixed(4)),
			};
			assert_eq_fromkdl!(EvaluatedRoll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn eval_amount() -> anyhow::Result<()> {
			let doc = "
				|roll {
				|    amount (Evaluator)\"get_proficiency_bonus\"
				|}
			";
			let data = EvaluatedRoll {
				amount: Value::Evaluated(GetProficiencyBonus.into()),
				die: None,
			};
			assert_eq_fromkdl!(EvaluatedRoll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn eval_die() -> anyhow::Result<()> {
			let doc = "
				|roll {
				|    amount 5
				|    die (Evaluator)\"get_proficiency_bonus\"
				|}
			";
			let data = EvaluatedRoll {
				amount: Value::Fixed(5),
				die: Some(Value::Evaluated(GetProficiencyBonus.into())),
			};
			assert_eq_fromkdl!(EvaluatedRoll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
