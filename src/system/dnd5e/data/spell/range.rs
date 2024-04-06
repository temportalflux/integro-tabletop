use crate::kdl_ext::NodeContext;
use crate::utility::NotInList;
use kdlize::{ext::ValueExt, AsKdl, FromKdl, NodeBuilder};

#[derive(Default, Clone, PartialEq, Debug)]
pub enum Range {
	#[default]
	OnlySelf,
	Touch,
	Unit {
		distance: u64,
		unit: String,
	},
	Sight,
	Unlimited,
}

impl FromKdl<NodeContext> for Range {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let entry = node.next_req()?;
		match entry.value().as_string() {
			None => {
				let distance = entry.as_i64_req()? as u64;
				let unit = node.next_str_opt()?.unwrap_or("Feet").to_owned();
				Ok(Self::Unit { distance, unit })
			}
			Some("Self") => Ok(Self::OnlySelf),
			Some("Touch") => Ok(Self::Touch),
			Some("Sight") => Ok(Self::Sight),
			Some("Unlimited") => Ok(Self::Unlimited),
			Some(type_name) => Err(NotInList(type_name.into(), vec!["Self", "Touch", "Sight", "Unlimited"]).into()),
		}
	}
}

impl AsKdl for Range {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::OnlySelf => node.with_entry("Self"),
			Self::Touch => node.with_entry("Touch"),
			Self::Sight => node.with_entry("Sight"),
			Self::Unlimited => node.with_entry("Unlimited"),
			Self::Unit { distance, unit } => {
				node.entry(*distance as i64);
				if unit != "Feet" {
					node.entry(unit.clone());
				}
				node
			}
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "range";

		#[test]
		fn only_self() -> anyhow::Result<()> {
			let doc = "range \"Self\"";
			let data = Range::OnlySelf;
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn touch() -> anyhow::Result<()> {
			let doc = "range \"Touch\"";
			let data = Range::Touch;
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn sight() -> anyhow::Result<()> {
			let doc = "range \"Sight\"";
			let data = Range::Sight;
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn unlimited() -> anyhow::Result<()> {
			let doc = "range \"Unlimited\"";
			let data = Range::Unlimited;
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn unit_feet() -> anyhow::Result<()> {
			let doc = "range 60";
			let data = Range::Unit {
				distance: 60,
				unit: "Feet".into(),
			};
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn unit_other() -> anyhow::Result<()> {
			let doc = "range 5 \"Miles\"";
			let data = Range::Unit {
				distance: 5,
				unit: "Miles".into(),
			};
			assert_eq_fromkdl!(Range, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
