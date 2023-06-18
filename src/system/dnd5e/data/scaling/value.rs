use super::{Basis, DefaultLevelMap};
use crate::{
	kdl_ext::{AsKdl, EntryExt, NodeBuilder, NodeExt},
	system::dnd5e::{data::character::Character, FromKDL},
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
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.entry_req(ctx.peak_idx())?.type_opt() {
			None => Ok(Self::Fixed(T::from_kdl(node, ctx)?)),
			Some("Scaled") => Ok(Self::Scaled(Basis::from_kdl(node, ctx)?)),
			Some(type_name) => Err(GeneralError(format!(
				"Invalid type name {type_name:?}, expected no type or Scaled."
			))
			.into()),
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
