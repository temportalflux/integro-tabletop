use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{core::NodeRegistry, dnd5e::FromKDL},
	GeneralError,
};
use enum_map::{Enum, EnumMap};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct BoundedValue(EnumMap<BoundKind, BTreeMap<PathBuf, i32>>);

impl<const N: usize> From<[(BoundKind, BTreeMap<PathBuf, i32>); N]> for BoundedValue {
	fn from(full_list: [(BoundKind, BTreeMap<PathBuf, i32>); N]) -> Self {
		let mut map = EnumMap::default();
		for (kind, entries) in full_list {
			map[kind] = entries;
		}
		Self(map)
	}
}

impl BoundedValue {
	pub fn insert(&mut self, arg: BoundValue, source: PathBuf) {
		let (kind, value) = arg.into_parts();
		self.0[kind].insert(source, value);
	}

	pub fn args(&self, kind: BoundKind) -> impl Iterator<Item = &i32> {
		self.0[kind].iter().map(|(_, value)| value)
	}

	pub fn value(&self) -> i32 {
		let minimum = self.args(BoundKind::Minimum).cloned().max();
		let additive: i32 = self.args(BoundKind::Additive).cloned().sum();
		let subtract: i32 = self.args(BoundKind::Subtract).cloned().sum();
		let total = additive - subtract;
		match minimum {
			None => total,
			Some(min_value) => total.max(min_value),
		}
	}

	pub fn argument(&self, kind: BoundKind) -> &BTreeMap<PathBuf, i32> {
		&self.0[kind]
	}

	pub fn iter(&self) -> impl Iterator<Item = (BoundKind, &PathBuf, &i32)> {
		self.0
			.iter()
			.map(|(kind, source_values)| {
				source_values
					.iter()
					.map(move |(path, value)| (kind, path, value))
			})
			.flatten()
	}
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Enum)]
pub enum BoundKind {
	Minimum,
	Additive,
	Subtract,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum BoundValue {
	Minimum(i32),
	Additive(i32),
	Subtract(i32),
}
impl BoundValue {
	pub fn kind(&self) -> BoundKind {
		match self {
			Self::Minimum(_) => BoundKind::Minimum,
			Self::Additive(_) => BoundKind::Additive,
			Self::Subtract(_) => BoundKind::Subtract,
		}
	}

	pub fn value(&self) -> &i32 {
		match self {
			Self::Minimum(v) => v,
			Self::Additive(v) => v,
			Self::Subtract(v) => v,
		}
	}

	pub fn into_value(self) -> i32 {
		match self {
			Self::Minimum(v) => v,
			Self::Additive(v) => v,
			Self::Subtract(v) => v,
		}
	}

	pub fn into_parts(self) -> (BoundKind, i32) {
		(self.kind(), self.into_value())
	}
}

impl FromKDL for BoundValue {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let entry_idx = value_idx.next();
		match node
			.entry_req(entry_idx)?
			.ty()
			.ok_or(GeneralError(format!(
				"Type id missing on value at index {entry_idx} of node {node:?}"
			)))?
			.value()
		{
			"Minimum" => Ok(Self::Minimum(node.get_i64(entry_idx)? as i32)),
			"Additive" => Ok(Self::Additive(node.get_i64(entry_idx)? as i32)),
			"Subtract" => Ok(Self::Subtract(node.get_i64(entry_idx)? as i32)),
			type_name => Err(GeneralError(format!(
				"Invalid bound value id {type_name:?}, \
				expected Minimum, Additive, or Subtract"
			))
			.into()),
		}
	}
}

// TODO: Tests for BoundValue::Subtract
#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn insert_minimum() {
		let mut sense = BoundedValue::default();
		sense.insert(BoundValue::Minimum(10), "FeatMin".into());
		assert_eq!(sense.0[BoundKind::Minimum], [("FeatMin".into(), 10)].into());
	}

	#[test]
	fn insert_additive() {
		let mut sense = BoundedValue::default();
		sense.insert(BoundValue::Additive(10), "FeatAdd".into());
		assert_eq!(
			sense.0[BoundKind::Additive],
			[("FeatAdd".into(), 10)].into()
		);
	}

	#[test]
	fn value_empty() {
		assert_eq!(BoundedValue::default().value(), 0);
	}

	#[test]
	fn value_min_only() {
		let mut sense = BoundedValue::default();
		sense.insert(BoundValue::Minimum(10), "FeatMin".into());
		assert_eq!(sense.value(), 10);
	}

	#[test]
	fn value_add_only() {
		let mut sense = BoundedValue::default();
		sense.0[BoundKind::Additive] = [("FeatAdd".into(), 20)].into();
		assert_eq!(sense.value(), 20);
	}

	#[test]
	fn value_add_multiple() {
		let mut sense = BoundedValue::default();
		sense.0[BoundKind::Additive] = [("A".into(), 5), ("B".into(), 10)].into();
		assert_eq!(sense.value(), 15);
	}

	#[test]
	fn value_min_gt_add() {
		let mut sense = BoundedValue::default();
		sense.0[BoundKind::Minimum] = [("A".into(), 20)].into();
		sense.0[BoundKind::Additive] = [("B".into(), 10)].into();
		assert_eq!(sense.value(), 20);
	}

	#[test]
	fn value_min_lt_add() {
		let mut sense = BoundedValue::default();
		sense.0[BoundKind::Minimum] = [("A".into(), 10)].into();
		sense.0[BoundKind::Additive] = [("B".into(), 20)].into();
		assert_eq!(sense.value(), 20);
	}

	#[test]
	fn value_min_lt_add_multiple() {
		let mut sense = BoundedValue::default();
		sense.0[BoundKind::Minimum] = [("A".into(), 60)].into();
		sense.0[BoundKind::Additive] = [("B".into(), 60), ("C".into(), 60)].into();
		assert_eq!(sense.value(), 120);
	}
}
