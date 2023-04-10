use crate::{
	kdl_ext::{FromKDL, NodeExt, ValueExt},
	system::dnd5e::{data::character::Character, Value},
	utility::{Dependencies, Evaluator, NotInList},
};

#[derive(thiserror::Error, Debug)]
#[error("Math operation \"Divide\" cannot support {0} values (max is 2).")]
pub struct DivideTooManyOps(usize);

#[derive(Clone, PartialEq, Debug)]
pub struct Math {
	pub operation: MathOp,
	pub minimum: Option<i32>,
	pub maximum: Option<i32>,
	pub values: Vec<Value<i32>>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum MathOp {
	Add,
	Subtract,
	Multiply,
	Divide { round: Rounding },
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Rounding {
	Floor,
	#[default]
	HalfUp,
	Ceiling,
}

crate::impl_trait_eq!(Math);
crate::impl_kdl_node!(Math, "math");

impl FromKDL for Math {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let operation = match node.get_str_req(ctx.consume_idx())? {
			"Add" => MathOp::Add,
			"Subtract" => MathOp::Subtract,
			"Multiply" => MathOp::Multiply,
			"Divide" => {
				let round = match node.get_str_opt("round")? {
					None => Rounding::default(),
					Some("ceil" | "ceiling" | "Ceiling") => Rounding::Ceiling,
					Some("floor" | "Floor") => Rounding::Floor,
					Some("halfup" | "HalfUp") => Rounding::HalfUp,
					Some(name) => {
						return Err(
							NotInList(name.into(), vec!["Floor", "HalfUp", "Ceiling"]).into()
						)
					}
				};
				MathOp::Divide { round }
			}
			name => {
				return Err(
					NotInList(name.into(), vec!["Add", "Subtract", "Multiply", "Divide"]).into(),
				)
			}
		};

		let minimum = node.get_i64_opt("min")?.map(|v| v as i32);
		let maximum = node.get_i64_opt("max")?.map(|v| v as i32);

		let mut values = Vec::new();
		for node in node.query_all("scope() > value")? {
			let mut ctx = ctx.next_node();
			values.push(Value::from_kdl(
				node,
				node.entry_req(ctx.consume_idx())?,
				&mut ctx,
				|value| Ok(value.as_i64_req()? as i32),
			)?);
		}
		match (&operation, values.len()) {
			(MathOp::Divide { .. }, len) if len > 2 => {
				return Err(DivideTooManyOps(len).into());
			}
			_ => {}
		}

		Ok(Self {
			operation,
			minimum,
			maximum,
			values,
		})
	}
}

impl Evaluator for Math {
	type Context = Character;
	type Item = i32;

	fn description(&self) -> Option<String> {
		let value_descriptions = self
			.values
			.iter()
			.filter_map(|value| value.description())
			.collect::<Vec<_>>();
		let description = match &self.operation {
			MathOp::Add => value_descriptions.join(" + "),
			MathOp::Subtract => value_descriptions.join(" - "),
			MathOp::Multiply => value_descriptions.join(" * "),
			MathOp::Divide { round } => format!(
				"{} {}",
				value_descriptions.join(" / "),
				match round {
					Rounding::Floor => "rounded down",
					Rounding::HalfUp => "rounded to the nearest whole number",
					Rounding::Ceiling => "rounded up",
				}
			),
		};
		let bounds = {
			let mut bounds = Vec::with_capacity(2);
			if let Some(min) = &self.minimum {
				bounds.push(format!("minimum {min}"));
			}
			if let Some(max) = &self.maximum {
				bounds.push(format!("maximum {max}"));
			}
			(!bounds.is_empty()).then(move || format!(" ({})", bounds.join(", ")))
		};
		Some(format!("{description}{}", bounds.unwrap_or_default()))
	}

	fn dependencies(&self) -> Dependencies {
		self.values
			.iter()
			.fold(Dependencies::default(), |deps, value| {
				deps.join(value.dependencies())
			})
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let mut iter_outputs = self.values.iter().map(|val| val.evaluate(state));
		let mut value = match &self.operation {
			MathOp::Add => iter_outputs.sum::<i32>(),
			MathOp::Subtract => {
				let mut value = iter_outputs.next().unwrap_or_default();
				for next in iter_outputs {
					value -= next;
				}
				value
			}
			MathOp::Multiply => iter_outputs.product::<i32>(),
			MathOp::Divide { round } => {
				let mut value = iter_outputs.next().unwrap_or_default() as f64;
				let next = iter_outputs.next().unwrap_or_default();
				if next != 0 {
					value /= next as f64;
				}
				match round {
					Rounding::Floor => value.floor() as i32,
					Rounding::Ceiling => value.ceil() as i32,
					Rounding::HalfUp => value.round() as i32,
				}
			}
		};
		if let Some(min) = self.minimum {
			value = value.max(min);
		}
		if let Some(max) = self.maximum {
			value = value.min(max);
		}
		value
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{
			system::{
				core::NodeRegistry,
				dnd5e::{
					data::Ability,
					evaluator::{GetAbilityModifier, GetLevel},
				},
			},
			utility::GenericEvaluator,
		};

		fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, i32>> {
			let mut reg = NodeRegistry::default();
			reg.register_evaluator::<Math>();
			reg.register_evaluator::<GetAbilityModifier>();
			reg.register_evaluator::<GetLevel>();
			reg.parse_kdl_evaluator(doc)
		}

		#[test]
		fn add() -> anyhow::Result<()> {
			let doc = "evaluator \"math\" \"Add\" max=15 {
				value 10
				value (Evaluator)\"get_ability_modifier\" (Ability)\"Strength\"
			}";
			let expected = Math {
				operation: MathOp::Add,
				minimum: None,
				maximum: Some(15),
				values: vec![
					Value::Fixed(10),
					Value::Evaluated(GetAbilityModifier(Ability::Strength).into()),
				],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn subtract() -> anyhow::Result<()> {
			let doc = "evaluator \"math\" \"Subtract\" min=0 {
				value (Evaluator)\"get_level\"
				value 10
			}";
			let expected = Math {
				operation: MathOp::Subtract,
				minimum: Some(0),
				maximum: None,
				values: vec![Value::Evaluated(GetLevel(None).into()), Value::Fixed(10)],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn multiply() -> anyhow::Result<()> {
			let doc = "evaluator \"math\" \"Multiply\" {
				value (Evaluator)\"get_ability_modifier\" (Ability)\"Constitution\"
				value (Evaluator)\"get_level\"
			}";
			let expected = Math {
				operation: MathOp::Multiply,
				minimum: None,
				maximum: None,
				values: vec![
					Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
					Value::Evaluated(GetLevel(None).into()),
				],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn divide() -> anyhow::Result<()> {
			let doc = "evaluator \"math\" \"Divide\" round=\"floor\" min=1 {
				value (Evaluator)\"get_level\"
				value 2
			}";
			let expected = Math {
				operation: MathOp::Divide {
					round: Rounding::Floor,
				},
				minimum: Some(1),
				maximum: None,
				values: vec![Value::Evaluated(GetLevel(None).into()), Value::Fixed(2)],
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::system::dnd5e::{
			data::{character::Persistent, Ability, Class, Level},
			evaluator::{GetAbilityModifier, GetLevel},
		};

		fn character(scores: &[(Ability, u32)], level: usize) -> Character {
			let mut persistent = Persistent::default();
			for (ability, score) in scores {
				persistent.ability_scores[*ability] = *score;
			}
			if level > 0 {
				persistent.classes.push(Class {
					name: "TestClass".into(),
					levels: {
						let mut vec = Vec::with_capacity(level);
						vec.resize_with(level, || Level::default());
						vec
					},
					..Default::default()
				});
			}
			Character::from(persistent)
		}

		#[test]
		fn add() {
			let math = Math {
				operation: MathOp::Add,
				minimum: None,
				maximum: Some(15),
				values: vec![
					Value::Fixed(10),
					Value::Evaluated(GetAbilityModifier(Ability::Strength).into()),
				],
			};
			// smaller than maximum
			let ctx = character(&[(Ability::Strength, 14)], 0);
			assert_eq!(math.evaluate(&ctx), 12);
			// larger than maximum
			let ctx = character(&[(Ability::Strength, 24)], 0);
			assert_eq!(math.evaluate(&ctx), 15);
		}

		#[test]
		fn subtract() {
			let math = Math {
				operation: MathOp::Subtract,
				minimum: Some(0),
				maximum: None,
				values: vec![Value::Evaluated(GetLevel(None).into()), Value::Fixed(10)],
			};
			// larger than minimum
			let ctx = character(&[], 12);
			assert_eq!(math.evaluate(&ctx), 2);
			// smaller than minimum
			let ctx = character(&[], 9);
			assert_eq!(math.evaluate(&ctx), 0);
		}

		#[test]
		fn multiply() {
			let math = Math {
				operation: MathOp::Multiply,
				minimum: None,
				maximum: None,
				values: vec![
					Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
					Value::Evaluated(GetLevel(None).into()),
				],
			};
			let ctx = character(&[(Ability::Constitution, 16)], 2);
			assert_eq!(math.evaluate(&ctx), 6);
		}

		#[test]
		fn divide_floor() {
			let math = Math {
				operation: MathOp::Divide {
					round: Rounding::Floor,
				},
				minimum: None,
				maximum: None,
				values: vec![Value::Evaluated(GetLevel(None).into()), Value::Fixed(4)],
			};
			let ctx = character(&[], 11);
			// 11 / 4 = 2.75 => floored = 2
			assert_eq!(math.evaluate(&ctx), 2);
		}

		#[test]
		fn divide_halfup() {
			let math = Math {
				operation: MathOp::Divide {
					round: Rounding::HalfUp,
				},
				minimum: None,
				maximum: None,
				values: vec![Value::Evaluated(GetLevel(None).into()), Value::Fixed(4)],
			};
			let ctx = character(&[], 11);
			// 11 / 4 = 2.75 => round up = 3
			assert_eq!(math.evaluate(&ctx), 3);
		}

		#[test]
		fn divide_ceiling() {
			let math = Math {
				operation: MathOp::Divide {
					round: Rounding::Ceiling,
				},
				minimum: None,
				maximum: None,
				values: vec![Value::Evaluated(GetLevel(None).into()), Value::Fixed(5)],
			};
			let ctx = character(&[], 11);
			// 11 / 5 = 2.2 => ceil = 3
			assert_eq!(math.evaluate(&ctx), 3);
		}
	}
}
