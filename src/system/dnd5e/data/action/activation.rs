use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{DnD5e, FromKDL},
	GeneralError,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub enum ActivationKind {
	#[default]
	Action,
	Bonus,
	Reaction,
	Minute(u32),
	Hour(u32),
}

impl FromKDL<DnD5e> for ActivationKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		match node.get_str(value_idx.next())? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			"Minute" => Ok(Self::Minute(node.get_i64(value_idx.next())? as u32)),
			"Hour" => Ok(Self::Hour(node.get_i64(value_idx.next())? as u32)),
			name => Err(GeneralError(format!(
				"Invalid action activation type {name:?}, expected \
				Action, Bonus, Reaction, Minute, or Hour."
			))
			.into()),
		}
	}
}
