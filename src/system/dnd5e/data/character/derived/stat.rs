use crate::{
	kdl_ext::{NodeContext, NodeReader},
	system::mutator::ReferencePath,
	utility::NotInList,
};
use kdlize::{
	ext::{EntryExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};
use multimap::MultiMap;
use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Stat {
	named: MultiMap<String, (StatOperation, PathBuf)>,
	global: Vec<(StatOperation, PathBuf)>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatOperation {
	// The calculated value can be no less than this value.
	MinimumValue(u32),
	// The calculated value can be no less than the value of this stat.
	// If the stat is not provided by other mutators, then this is ignored / the minimum is zero.
	MinimumStat(String),
	// The calculated value starts with this value, and can be adjusted by other numerical changes.
	// If there are multiple Base values, the largest is used.
	Base(u32),
	// The base value gets multiplied by (if zero or positive) or divided by (if negative) this value.
	MultiplyDivide(i32),
	// The base value gets added or subtracted by this value.
	AddSubtract(i32),
	// The value calculated after minimums, base, and multiplication and addition is capped by this value
	MaximumValue(u32),
}

impl FromKdl<NodeContext> for StatOperation {
	type Error = anyhow::Error;
	fn from_kdl(node: &mut NodeReader) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		match entry.type_req()? {
			"Minimum" => match entry.value().as_string() {
				Some(value) => Ok(Self::MinimumStat(value.to_owned())),
				None => Ok(Self::MinimumValue(entry.as_i64_req()? as u32)),
			},
			"Base" => Ok(Self::Base(entry.as_i64_req()? as u32)),
			"Additive" | "Add" => {
				let value = entry.as_i64_req()? as i32;
				Ok(Self::AddSubtract(value.abs()))
			}
			"Subtract" => {
				let value = entry.as_i64_req()? as i32;
				Ok(Self::AddSubtract(-value.abs()))
			}
			"Multiply" => {
				let value = entry.as_i64_req()? as i32;
				Ok(Self::MultiplyDivide(value.abs()))
			}
			"Divide" => {
				let value = entry.as_i64_req()? as i32;
				if value == 0 {
					return Err(anyhow::Error::msg("Invalid StatOperation (Divide)0"));
				}
				Ok(Self::MultiplyDivide(-value.abs()))
			}
			"Maximum" => Ok(Self::MaximumValue(entry.as_i64_req()? as u32)),
			type_str => Err(NotInList(type_str.into(), vec![
				"Minimum", "Base", "Add", "Subtract", "Multiply", "Divide", "Maximum",
			]))?,
		}
	}
}

impl AsKdl for StatOperation {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::MinimumValue(value) => node.with_entry_typed(*value as i64, "Minimum"),
			Self::MinimumStat(stat_name) => node.with_entry_typed(stat_name.as_str(), "Minimum"),
			Self::Base(value) => node.with_entry_typed(*value as i64, "Base"),
			Self::AddSubtract(value) if *value >= 0 => node.with_entry_typed(value.abs() as i64, "Add"),
			Self::AddSubtract(value) => node.with_entry_typed(value.abs() as i64, "Subtract"),
			Self::MultiplyDivide(value) if *value >= 0 => node.with_entry_typed(value.abs() as i64, "Multiply"),
			Self::MultiplyDivide(value) => node.with_entry_typed(value.abs() as i64, "Divide"),
			Self::MaximumValue(value) => node.with_entry_typed(*value as i64, "Maximum"),
		}
	}
}

impl Stat {
	pub fn insert(&mut self, kind: Option<String>, operation: StatOperation, source: &ReferencePath) {
		match kind {
			None => {
				self.global.push((operation, source.display.clone()));
			}
			Some(name) => {
				self.named.insert(name, (operation, source.display.clone()));
			}
		}
	}

	pub fn len(&self) -> usize {
		self.names().count()
	}

	pub fn names(&self) -> impl Iterator<Item = &String> + '_ {
		self.named.keys()
	}

	pub fn iter_compiled(&self) -> impl Iterator<Item = (&str, u32)> + '_ {
		let iter = self.named.iter_all();
		// For each set of operations keyed by stat name, compile the expected value and the list of stats that it must be at least equivalent to
		let iter = iter.map(|(stat_name, operations)| {
			let iter_ops = operations.iter().chain(self.global.iter()).map(|(stat_op, _path)| stat_op);
			let (value, minimum_sibling_stats) = Self::compile_stat(iter_ops);
			((stat_name.as_str(), value), (stat_name.as_str(), minimum_sibling_stats))
		});
		// split so that there is a map of StatName->Value and StatName->[Other Stat Names]
		let (mut stat_values, stat_minimum_sibling_stats): (HashMap<_, _>, HashMap<_, _>) = iter.unzip();
		// For all stats which are at minimum equivalent to some other stat
		for (stat_name, minimum_sibling_stats) in stat_minimum_sibling_stats {
			// Find the maximum of all the minimum values, where each minimum value is found by the stat's name
			let iter = minimum_sibling_stats.into_iter().filter_map(|stat_name| stat_values.get(stat_name).copied());
			if let Some(minimum_value) = iter.max() {
				let value = stat_values.get_mut(stat_name).expect("missing valid key");
				*value = (*value).min(minimum_value);
			}
		}
		stat_values.into_iter()
	}

	pub fn get(&self, stat_name: impl AsRef<str>) -> impl Iterator<Item = &(StatOperation, PathBuf)> + '_ {
		let entry = self.named.get_vec(stat_name.as_ref());
		let entry = entry.map(|values| values.iter());
		let iter = entry.unwrap_or_default();
		let iter = iter.chain(self.global.iter());
		iter
	}

	fn compile_stat<'iter>(operations: impl Iterator<Item = &'iter StatOperation>) -> (u32, Vec<&'iter str>) {
		let mut minimum_value = 0u32;
		let mut minimum_stat_match = Vec::new();
		let mut maximum_base = 0u32;
		let mut linear_diff = 0i32;
		let mut multipier = 1u32;
		let mut divisor = 1u32;
		let mut maximum_value = u32::MAX;
		for operation in operations {
			match operation {
				StatOperation::MinimumValue(value) => minimum_value = minimum_value.min(*value),
				StatOperation::MinimumStat(stat_name) => minimum_stat_match.push(stat_name.as_str()),
				StatOperation::Base(value) => maximum_base = maximum_base.max(*value),
				StatOperation::AddSubtract(value) => linear_diff += *value,
				StatOperation::MultiplyDivide(value) => match value.cmp(&0) {
					// NOTE: intentional mathematical discrepancy, a multiplier of 0 is ignored (treated as +1)
					Ordering::Equal => {}
					Ordering::Greater => multipier *= value.abs() as u32,
					Ordering::Less => divisor *= value.abs() as u32,
				},
				StatOperation::MaximumValue(value) => {
					maximum_value = maximum_value.min(*value);
				}
			}
		}
		let value = maximum_base.saturating_mul(multipier);
		let value = value.saturating_div(divisor);
		let value = value.saturating_add_signed(linear_diff);
		let value = value.max(minimum_value);
		let value = value.min(maximum_value);
		(value, minimum_stat_match)
	}
}
