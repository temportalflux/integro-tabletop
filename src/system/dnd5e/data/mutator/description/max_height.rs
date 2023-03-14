use crate::{
	kdl_ext::ValueIdx,
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{character::Character, roll::Roll},
			FromKDL, KDLNode,
		},
	},
	utility::Mutator,
	GeneralError,
};
use std::str::FromStr;

#[derive(Clone, PartialEq)]
pub enum AddMaxHeight {
	Value(i32),
	Roll(Roll),
}

crate::impl_trait_eq!(AddMaxHeight);
impl std::fmt::Debug for AddMaxHeight {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Value(v) => write!(f, "AddMaxHeight(Value:{v})"),
			Self::Roll(r) => write!(f, "AddMaxHeight(Roll:{})", r.to_string()),
		}
	}
}

crate::impl_kdl_node!(AddMaxHeight, "add_max_height");

impl Mutator for AddMaxHeight {
	type Target = Character;

	fn apply<'c>(&self, stats: &mut Character) {
		match self {
			Self::Value(value) => {
				stats.derived_description_mut().max_height.0 += *value;
			}
			Self::Roll(roll) => {
				stats.derived_description_mut().max_height.1.push(*roll);
			}
		}
	}
}

impl FromKDL for AddMaxHeight {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let idx_val = value_idx.next();
		let entry = node.entry(idx_val).ok_or(GeneralError(format!(
			"Missing value at index {idx_val} for {:?}",
			Self::id()
		)))?;
		let type_name = entry.ty().map(|id| id.value());
		if type_name == Some("Roll") || entry.value().is_string_value() {
			let value = entry
				.value()
				.as_string()
				.ok_or(GeneralError("Roll value must be a string".into()))?;
			return Ok(Self::Roll(Roll::from_str(value)?));
		} else {
			let value = entry.value().as_i64().ok_or(GeneralError(format!(
				"Value for {:?} at index {idx_val} must either be a Roll or an integer",
				Self::id()
			)))?;
			Ok(Self::Value(value as i32))
		}
	}
}

// TODO: Test AddMaxHeight
