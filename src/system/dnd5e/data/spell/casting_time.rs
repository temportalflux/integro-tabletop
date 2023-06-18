use crate::kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt};

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

impl FromKDL for CastingTime {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let duration = CastingDuration::from_kdl(node, ctx)?;
		let ritual = node.get_bool_opt("ritual")?.unwrap_or_default();
		Ok(Self { duration, ritual })
	}
}

impl AsKdl for CastingTime {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.duration.as_kdl();
		if self.ritual {
			node.push_entry(("ritual", true));
		}
		node
	}
}

impl FromKDL for CastingDuration {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Action" => Ok(Self::Action),
			"Bonus" => Ok(Self::Bonus),
			"Reaction" => Ok(Self::Reaction(
				node.get_str_opt(ctx.consume_idx())?.map(str::to_owned),
			)),
			unit => Ok(Self::Unit(
				node.get_i64_req(ctx.consume_idx())? as u64,
				unit.to_owned(),
			)),
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
					node.push_entry(ctx.clone());
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
