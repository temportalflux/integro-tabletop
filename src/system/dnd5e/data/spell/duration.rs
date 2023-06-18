use crate::kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeContext, NodeExt};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Duration {
	pub concentration: bool,
	pub kind: DurationKind,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum DurationKind {
	#[default]
	Instantaneous,
	Unit(u64, String),
	Special,
}

impl FromKDL for Duration {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let kind = DurationKind::from_kdl(node, ctx)?;
		let concentration = node.get_bool_opt("concentration")?.unwrap_or_default();
		Ok(Self {
			concentration,
			kind,
		})
	}
}

impl AsKdl for Duration {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.kind.as_kdl();
		if self.concentration {
			node.push_entry(("concentration", true));
		}
		node
	}
}

impl FromKDL for DurationKind {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Instantaneous" => Ok(Self::Instantaneous),
			"Special" => Ok(Self::Special),
			unit => {
				let distance = node.get_i64_req(ctx.consume_idx())? as u64;
				Ok(Self::Unit(distance, unit.to_owned()))
			}
		}
	}
}

impl AsKdl for DurationKind {
	fn as_kdl(&self) -> NodeBuilder {
		let node = NodeBuilder::default();
		match self {
			Self::Instantaneous => node.with_entry("Instantaneous"),
			Self::Special => node.with_entry("Special"),
			Self::Unit(distance, unit) => {
				node.with_entry(unit.clone()).with_entry(*distance as i64)
			}
		}
	}
}

impl DurationKind {
	pub fn as_metadata(&self) -> String {
		match self {
			Self::Instantaneous => "instant",
			Self::Special | Self::Unit(_, _) => "other",
		}
		.into()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::kdl_ext::test_utils::*;

		static NODE_NAME: &str = "duration";

		#[test]
		fn instant() -> anyhow::Result<()> {
			let doc = "duration \"Instantaneous\"";
			let data = Duration {
				kind: DurationKind::Instantaneous,
				concentration: false,
			};
			assert_eq_fromkdl!(Duration, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn instant_concentration() -> anyhow::Result<()> {
			let doc = "duration \"Instantaneous\" concentration=true";
			let data = Duration {
				kind: DurationKind::Instantaneous,
				concentration: true,
			};
			assert_eq_fromkdl!(Duration, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn special() -> anyhow::Result<()> {
			let doc = "duration \"Special\"";
			let data = Duration {
				kind: DurationKind::Special,
				concentration: false,
			};
			assert_eq_fromkdl!(Duration, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn unit() -> anyhow::Result<()> {
			let doc = "duration \"Minute\" 10";
			let data = Duration {
				kind: DurationKind::Unit(10, "Minute".into()),
				concentration: false,
			};
			assert_eq_fromkdl!(Duration, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
