use crate::kdl_ext::NodeContext;
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct CastingTime {
	pub duration: CastingDuration,
	pub ritual: bool,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum CastingDuration {
	#[default]
	Action,
	Bonus,
	Reaction(Option<String>),
	Unit(u64, String),
}

impl FromKdl<NodeContext> for CastingTime {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let duration = CastingDuration::from_kdl(node)?;
		let ritual = node.get_bool_opt("ritual")?.unwrap_or_default();
		Ok(Self { duration, ritual })
	}
}

impl AsKdl for CastingTime {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.duration.as_kdl();
		if self.ritual {
			node.entry(("ritual", true));
		}
		node
	}
}

impl FromKdl<NodeContext> for CastingDuration {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction(node.next_str_opt()?.map(str::to_owned))),
			unit => Ok(Self::Unit(node.next_i64_req()? as u64, unit.to_owned())),
		}
	}
}

impl AsKdl for CastingDuration {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Action => node.with_entry("Action"),
			Self::Bonus => node.with_entry("Bonus"),
			Self::Reaction(ctx) => {
				let mut node = node.with_entry("Reaction");
				if let Some(ctx) = ctx {
					node.entry(ctx.clone());
				}
				node
			}
			Self::Unit(amt, unit) => node.with_entry(unit.clone()).with_entry(*amt as i64),
		}
	}
}

impl CastingDuration {
	pub fn as_metadata(&self) -> String {
		match self {
			Self::Action => "action",
			Self::Bonus => "bonus",
			Self::Reaction(_) => "reaction",
			Self::Unit(_, _) => "other",
		}
		.into()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;

		mod time {
			use super::*;
			use crate::kdl_ext::test_utils::*;

			static NODE_NAME: &str = "casting-time";

			#[test]
			fn duration() -> anyhow::Result<()> {
				let doc = "casting-time \"Action\"";
				let data = CastingTime {
					duration: CastingDuration::Action,
					ritual: false,
				};
				assert_eq_fromkdl!(CastingTime, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn ritual() -> anyhow::Result<()> {
				let doc = "casting-time \"Action\" ritual=true";
				let data = CastingTime {
					duration: CastingDuration::Action,
					ritual: true,
				};
				assert_eq_fromkdl!(CastingTime, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}
		}

		mod duration {
			use super::*;
			use crate::kdl_ext::test_utils::*;

			static NODE_NAME: &str = "duration";

			#[test]
			fn action() -> anyhow::Result<()> {
				let doc = "duration \"Action\"";
				let data = CastingDuration::Action;
				assert_eq_fromkdl!(CastingDuration, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn bonus() -> anyhow::Result<()> {
				let doc = "duration \"Bonus\"";
				let data = CastingDuration::Bonus;
				assert_eq_fromkdl!(CastingDuration, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn reaction() -> anyhow::Result<()> {
				let doc = "duration \"Reaction\"";
				let data = CastingDuration::Reaction(None);
				assert_eq_fromkdl!(CastingDuration, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn reaction_context() -> anyhow::Result<()> {
				let doc = "duration \"Reaction\" \"when falling\"";
				let data = CastingDuration::Reaction(Some("when falling".into()));
				assert_eq_fromkdl!(CastingDuration, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}

			#[test]
			fn unit() -> anyhow::Result<()> {
				let doc = "duration \"Minute\" 10";
				let data = CastingDuration::Unit(10, "Minute".into());
				assert_eq_fromkdl!(CastingDuration, doc, data);
				assert_eq_askdl!(&data, doc);
				Ok(())
			}
		}
	}
}
