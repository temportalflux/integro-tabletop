use super::{Basis, DefaultLevelMap};
use crate::kdl_ext::NodeContext;
use crate::{system::dnd5e::data::character::Character, GeneralError};
use kdlize::{ext::EntryExt, AsKdl, FromKdl, NodeBuilder};

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

impl<T> FromKdl<NodeContext> for Value<T>
where
	T: Clone + DefaultLevelMap + FromKdl<NodeContext>,
	anyhow::Error: From<T::Error>,
{
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.peak_req()?.type_opt() {
			None => Ok(Self::Fixed(T::from_kdl(node)?)),
			Some("Scaled") => Ok(Self::Scaled(Basis::from_kdl(node)?)),
			Some(type_name) => {
				Err(GeneralError(format!("Invalid type name {type_name:?}, expected no type or Scaled.")).into())
			}
		}
	}
}
impl<T> AsKdl for Value<T>
where
	T: Clone + DefaultLevelMap + AsKdl,
{
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Fixed(v) => v.as_kdl().without_type(),
			Self::Scaled(basis) => {
				let mut node = basis.as_kdl();
				node.set_first_entry_ty("Scaled");
				node
			}
		}
	}
}
