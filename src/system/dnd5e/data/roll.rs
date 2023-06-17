use super::character::Character;
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeExt, ValueExt},
	system::dnd5e::Value,
	GeneralError,
};
use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Roll {
	amount: u32,
	die: Option<Die>,
}
impl From<u32> for Roll {
	fn from(amount: u32) -> Self {
		Self { amount, die: None }
	}
}
impl From<(u32, Die)> for Roll {
	fn from((amount, die): (u32, Die)) -> Self {
		Self {
			amount,
			die: Some(die),
		}
	}
}
impl ToString for Roll {
	fn to_string(&self) -> String {
		match self.die {
			None => self.amount.to_string(),
			Some(die) => format!("{}d{}", self.amount, die.value()),
		}
	}
}
impl Roll {
	pub fn min(&self) -> u32 {
		self.amount
	}

	pub fn max(&self) -> u32 {
		self.amount * self.die.clone().map(Die::value).unwrap_or(1)
	}

	pub fn as_nonzero_string(&self) -> Option<String> {
		(self.amount > 0).then(|| self.to_string())
	}

	pub fn roll(&self, rand: &mut impl rand::Rng) -> u32 {
		match self.die {
			None => self.amount,
			Some(die) => die.roll(rand, self.amount),
		}
	}
}
impl FromStr for Roll {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		static EXPECTED: &'static str = "{int}d{int}";
		if !s.contains('d') {
			let amount = s.parse::<u32>()?;
			return Ok(Self::from(amount));
		}
		let mut parts = s.split('d');
		let amount_str = parts.next().ok_or(GeneralError(format!(
			"Roll string {s:?} missing amount, expected format {EXPECTED:?}."
		)))?;
		let die_str = parts.next().ok_or(GeneralError(format!(
			"Roll string {s:?} missing die type, expected format {EXPECTED:?}."
		)))?;
		if parts.next().is_some() {
			return Err(GeneralError(format!(
				"Too many parts in {s:?} for Roll, expected {EXPECTED:?}"
			))
			.into());
		}
		let amount = amount_str.parse::<u32>()?;
		let die = Die::try_from(die_str.parse::<u32>()?)?;
		Ok(Self {
			amount,
			die: Some(die),
		})
	}
}
// TODO AsKdl: from/as tests for Roll
impl FromKDL for Roll {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self::from_str(node.get_str_req(ctx.consume_idx())?)?)
	}
}
impl AsKdl for Roll {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default().with_entry(self.to_string())
	}
}
impl Roll {
	pub fn from_kdl_value(kdl: &kdl::KdlValue) -> anyhow::Result<Self> {
		if let Some(amt) = kdl.as_i64() {
			return Ok(Self::from(amt as u32));
		}
		if let Some(str) = kdl.as_string() {
			return Ok(Self::from_str(str)?);
		}
		Err(crate::kdl_ext::InvalidValueType(kdl.clone(), "i64 or string").into())
	}
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct RollSet(EnumMap<Die, u32>, u32);

impl RollSet {
	pub fn multiple(roll: &Roll, amount: u32) -> Self {
		let mut set = Self::default();
		match &roll.die {
			None => set.1 += roll.amount * amount,
			Some(die) => {
				set.0[*die] += roll.amount * amount;
			}
		}
		set
	}

	pub fn push(&mut self, roll: Roll) {
		match roll.die {
			None => self.1 += roll.amount,
			Some(die) => {
				self.0[die] += roll.amount;
			}
		}
	}

	pub fn extend(&mut self, set: RollSet) {
		for (die, amt) in set.0 {
			self.0[die] += amt;
		}
		self.1 += set.1;
	}

	pub fn rolls(&self) -> Vec<Roll> {
		let mut rolls = Vec::with_capacity(Die::LENGTH + 1);
		for (die, amt) in &self.0 {
			if *amt == 0 {
				continue;
			}
			rolls.push(Roll::from((*amt, die)));
		}
		if self.1 > 0 {
			rolls.push(Roll::from(self.1));
		}
		rolls
	}

	pub fn min(&self) -> u32 {
		let mut value = self.1;
		for (_die, amt) in &self.0 {
			if *amt > 0 {
				value += *amt;
			}
		}
		value
	}

	pub fn max(&self) -> u32 {
		let mut value = self.1;
		for (die, amt) in &self.0 {
			if *amt > 0 {
				value += *amt * die.value();
			}
		}
		value
	}

	pub fn as_nonzero_string(&self) -> Option<String> {
		let roll_strs = self
			.rolls()
			.iter()
			.filter_map(Roll::as_nonzero_string)
			.collect::<Vec<_>>();
		(!roll_strs.is_empty()).then(|| roll_strs.join(" + "))
	}

	pub fn roll(&self, rand: &mut impl rand::Rng) -> u32 {
		let mut value = self.1;
		for (die, amt) in &self.0 {
			value += die.roll(rand, *amt);
		}
		value
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct EvaluatedRoll {
	amount: Value<i32>,
	die: Option<Value<i32>>,
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
impl FromKDL for EvaluatedRoll {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		if let Some(roll_str) = node.get_str_opt(ctx.consume_idx())? {
			return Ok(Self::from(Roll::from_str(roll_str)?));
		}
		let amount = {
			let node = node.query_req("scope() > amount")?;
			let mut ctx = ctx.next_node();
			Value::from_kdl(
				node,
				node.entry_req(ctx.consume_idx())?,
				&mut ctx,
				|value| Ok(value.as_i64_req()? as i32),
			)?
		};
		let die = match node.query_opt("scope() > die")? {
			None => None,
			Some(node) => {
				let mut ctx = ctx.next_node();
				Some(Value::from_kdl(
					node,
					node.entry_req(ctx.consume_idx())?,
					&mut ctx,
					|value| Ok(value.as_i64_req()? as i32),
				)?)
			}
		};
		Ok(Self { amount, die })
	}
}
// TODO AsKdl: from/as tests for EvaluatedRoll
impl AsKdl for EvaluatedRoll {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			// These first two are when the EvaluatedRoll is a fixed Roll, and thus can be serialized as such
			Self {
				amount: Value::Fixed(amt),
				die: None,
			} => node.with_entry(format!("{amt}")),
			Self {
				amount: Value::Fixed(amt),
				die: Some(Value::Fixed(die)),
			} => node.with_entry(format!("{amt}d{die}")),
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

#[derive(Debug, Enum, EnumSetType, Default)]
pub enum Die {
	#[default]
	D4,
	D6,
	D8,
	D10,
	D12,
	D20,
}
impl Die {
	pub fn value(self) -> u32 {
		match self {
			Self::D4 => 4,
			Self::D6 => 6,
			Self::D8 => 8,
			Self::D10 => 10,
			Self::D12 => 12,
			Self::D20 => 20,
		}
	}

	pub fn roll(&self, rand: &mut impl rand::Rng, num: u32) -> u32 {
		if num == 0 {
			return 0;
		}
		let range = 1..=self.value();
		(0..num).map(|_| rand.gen_range(range.clone())).sum()
	}
}
impl TryFrom<u32> for Die {
	type Error = GeneralError;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		match value {
			4 => Ok(Self::D4),
			6 => Ok(Self::D6),
			8 => Ok(Self::D8),
			10 => Ok(Self::D10),
			12 => Ok(Self::D12),
			20 => Ok(Self::D20),
			_ => Err(GeneralError(format!("Invalid die number: {value}"))),
		}
	}
}
impl FromStr for Die {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"d4" => Ok(Self::D4),
			"d6" => Ok(Self::D6),
			"d8" => Ok(Self::D8),
			"d10" => Ok(Self::D10),
			"d12" => Ok(Self::D12),
			"d20" => Ok(Self::D20),
			_ => Err(GeneralError(format!(
				"Invalid die type {s:?}, expected d4, d6, d8, d10, d12, or d20"
			))),
		}
	}
}
impl ToString for Die {
	fn to_string(&self) -> String {
		match self {
			Self::D4 => "d4",
			Self::D6 => "d6",
			Self::D8 => "d8",
			Self::D10 => "d10",
			Self::D12 => "d12",
			Self::D20 => "d20",
		}
		.to_owned()
	}
}

#[derive(Debug, Enum, EnumSetType, PartialOrd, Ord)]
pub enum Modifier {
	Advantage,
	Disadvantage,
}
impl Modifier {
	pub fn display_name(&self) -> &'static str {
		match self {
			Modifier::Advantage => "Advantage",
			Modifier::Disadvantage => "Disadvantage",
		}
	}
}
impl FromStr for Modifier {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Advantage" => Ok(Self::Advantage),
			"Disadvantage" => Ok(Self::Disadvantage),
			_ => Err(GeneralError(format!(
				"Invalid roll modifier value {s:?}, expected Advantage or Disadvantage."
			))),
		}
	}
}
