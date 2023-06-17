use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	utility::NotInList,
};
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub enum ActivationKind {
	#[default]
	Action,
	Bonus,
	Reaction,
	Special,
	Minute(u32),
	Hour(u32),
}

impl ToString for ActivationKind {
	fn to_string(&self) -> String {
		match self {
			Self::Action => "Action".to_owned(),
			Self::Bonus => "Bonus Action".to_owned(),
			Self::Reaction => "Reaction".to_owned(),
			Self::Special => "Special".to_owned(),
			Self::Minute(amt) => format!("{amt} Minutes"),
			Self::Hour(amt) => format!("{amt} Hours"),
		}
	}
}

impl FromStr for ActivationKind {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			"Special" => Ok(Self::Special),
			name => Err(NotInList(
				name.into(),
				vec!["Action", "Bonus", "Reaction", "Special"],
			)),
		}
	}
}

impl FromKDL for ActivationKind {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			"Special" => Ok(Self::Special),
			"Minute" => Ok(Self::Minute(node.get_i64_req(ctx.consume_idx())? as u32)),
			"Hour" => Ok(Self::Hour(node.get_i64_req(ctx.consume_idx())? as u32)),
			name => Err(NotInList(
				name.into(),
				vec!["Action", "Bonus", "Reaction", "Special", "Minute", "Hour"],
			)
			.into()),
		}
	}
}
// TODO AsKdl: from/as tests for ActivationKind
impl AsKdl for ActivationKind {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Action => node.with_entry("Action"),
			Self::Bonus => node.with_entry("Bonus"),
			Self::Reaction => node.with_entry("Reaction"),
			Self::Special => node.with_entry("Special"),
			Self::Minute(amt) => node.with_entry("Minute").with_entry(*amt as i64),
			Self::Hour(amt) => node.with_entry("Hour").with_entry(*amt as i64),
		}
	}
}
