use super::{Basis, DefaultLevelMap};
use crate::{
	kdl_ext::{EntryExt, NodeExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::character::Character, FromKDL},
	},
	GeneralError,
};

#[derive(Clone, PartialEq, Debug)]
pub enum Value<T> {
	Fixed(T),
	Scaled(Basis<T>),
}

impl<T> Default for Value<T>
where
	T: Default,
{
	fn default() -> Self {
		Self::Fixed(T::default())
	}
}

impl<T> Value<T>
where
	T: Clone + DefaultLevelMap,
{
	pub fn evaluate(&self, character: &Character) -> Option<T> {
		match self {
			Self::Fixed(value) => Some(value.clone()),
			Self::Scaled(basis) => basis.evaluate(character),
		}
	}
}

impl<T> FromKDL for Value<T>
where
	T: Clone + DefaultLevelMap + FromKDL,
{
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		match node.entry_req(**value_idx)?.type_opt() {
			None => Ok(Self::Fixed(T::from_kdl(node, value_idx, node_reg)?)),
			Some("Scaled") => Ok(Self::Scaled(Basis::from_kdl(node, value_idx, node_reg)?)),
			Some(type_name) => Err(GeneralError(format!(
				"Invalid type name {type_name:?}, expected no type or Scaled."
			))
			.into()),
		}
	}
}
