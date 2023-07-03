use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
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
	fn from_kdl_reader<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction),
			"Special" => Ok(Self::Special),
			"Minute" => Ok(Self::Minute(node.next_i64_req()? as u32)),
			"Hour" => Ok(Self::Hour(node.next_i64_req()? as u32)),
			name => Err(NotInList(
				name.into(),
				vec!["Action", "Bonus", "Reaction", "Special", "Minute", "Hour"],
			)
			.into()),
		}
	}
}

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

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "kind";

		#[test]
		fn action() -> anyhow::Result<()> {
			let doc = "kind \"Action\"";
			let data = ActivationKind::Action;
			assert_eq_fromkdl!(ActivationKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn bonus() -> anyhow::Result<()> {
			let doc = "kind \"Bonus\"";
			let data = ActivationKind::Bonus;
			assert_eq_fromkdl!(ActivationKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn reaction() -> anyhow::Result<()> {
			let doc = "kind \"Reaction\"";
			let data = ActivationKind::Reaction;
			assert_eq_fromkdl!(ActivationKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn special() -> anyhow::Result<()> {
			let doc = "kind \"Special\"";
			let data = ActivationKind::Special;
			assert_eq_fromkdl!(ActivationKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn minute() -> anyhow::Result<()> {
			let doc = "kind \"Minute\" 5";
			let data = ActivationKind::Minute(5);
			assert_eq_fromkdl!(ActivationKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn hour() -> anyhow::Result<()> {
			let doc = "kind \"Hour\" 1";
			let data = ActivationKind::Hour(1);
			assert_eq_fromkdl!(ActivationKind, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
