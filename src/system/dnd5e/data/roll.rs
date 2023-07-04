use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	GeneralError,
};
use std::str::FromStr;

mod die;
pub use die::*;
mod evaluated;
pub use evaluated::*;
mod modifier;
pub use modifier::*;
mod set;
pub use set::*;

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
	type Err = ParseRollError;

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

#[derive(thiserror::Error, Debug)]
pub enum ParseRollError {
	#[error(transparent)]
	ParseInt(#[from] std::num::ParseIntError),
	#[error(transparent)]
	Format(#[from] GeneralError),
}

impl FromKDL for Roll {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(node.next_str_req_t::<Self>()?)
	}
}

impl AsKdl for Roll {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		if self.die.is_none() {
			node.with_entry(self.to_string())
		} else {
			node.with_entry_typed(self.to_string(), "Roll")
		}
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

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "roll";

		#[test]
		fn fixed() -> anyhow::Result<()> {
			let doc = "roll \"4\"";
			let data = Roll::from(4);
			assert_eq_fromkdl!(Roll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn single_die() -> anyhow::Result<()> {
			let doc = "roll (Roll)\"1d6\"";
			let data = Roll::from((1, Die::D6));
			assert_eq_fromkdl!(Roll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn multi_die() -> anyhow::Result<()> {
			let doc = "roll (Roll)\"8d10\"";
			let data = Roll::from((8, Die::D10));
			assert_eq_fromkdl!(Roll, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
