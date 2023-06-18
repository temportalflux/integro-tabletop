use crate::{
	kdl_ext::{AsKdl, FromKDL, KDLNode, NodeBuilder, NodeExt},
	system::dnd5e::{
		data::character::{Character, HitPoint},
		mutator::AddMaxHitPoints,
	},
	utility::{Dependencies, Evaluator},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct GetHitPoints(pub HitPoint);

crate::impl_trait_eq!(GetHitPoints);
crate::impl_kdl_node!(GetHitPoints, "get_hit_points");

impl FromKDL for GetHitPoints {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(HitPoint::from_str(
			node.get_str_req(ctx.consume_idx())?,
		)?))
	}
}

impl AsKdl for GetHitPoints {
	fn as_kdl(&self) -> NodeBuilder {
		NodeBuilder::default().with_entry(match self.0 {
			HitPoint::Current => "Current",
			HitPoint::Temp => "Temp",
			HitPoint::Max => "Max",
		})
	}
}

impl Evaluator for GetHitPoints {
	type Context = Character;
	type Item = i32;

	fn description(&self) -> Option<String> {
		Some(
			match self.0 {
				HitPoint::Current => "your current hit points",
				HitPoint::Temp => "your temporary hit points",
				HitPoint::Max => "your hit point maximum",
			}
			.into(),
		)
	}

	fn dependencies(&self) -> Dependencies {
		match self.0 {
			HitPoint::Max => [AddMaxHitPoints::id()].into(),
			_ => Dependencies::default(),
		}
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		state.get_hp(self.0) as i32
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::evaluator::test::test_utils};

		test_utils!(GetHitPoints);

		#[test]
		fn current() -> anyhow::Result<()> {
			let doc = "evaluator \"get_hit_points\" \"Current\"";
			let data = GetHitPoints(HitPoint::Current);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn temp() -> anyhow::Result<()> {
			let doc = "evaluator \"get_hit_points\" \"Temp\"";
			let data = GetHitPoints(HitPoint::Temp);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn max() -> anyhow::Result<()> {
			let doc = "evaluator \"get_hit_points\" \"Max\"";
			let data = GetHitPoints(HitPoint::Max);
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::{
			system::dnd5e::data::{character::Persistent, Bundle},
			utility::Value,
		};

		fn character(current: u32, temp: u32, max: u32) -> Character {
			let mut persistent = Persistent::default();
			persistent.hit_points.current = current;
			persistent.hit_points.temp = temp;
			if max > 0 {
				persistent.bundles.push(
					Bundle {
						name: "MaxHP".into(),
						mutators: vec![AddMaxHitPoints {
							id: None,
							value: Value::Fixed(max as i32),
						}
						.into()],
						..Default::default()
					}
					.into(),
				);
			}
			Character::from(persistent)
		}

		#[test]
		fn current() {
			let eval = GetHitPoints(HitPoint::Current);
			let ctx = character(23, 5, 45);
			assert_eq!(eval.evaluate(&ctx), 23);
		}

		#[test]
		fn temp() {
			let eval = GetHitPoints(HitPoint::Temp);
			let ctx = character(23, 5, 45);
			assert_eq!(eval.evaluate(&ctx), 5);
		}

		#[test]
		fn max() {
			let eval = GetHitPoints(HitPoint::Max);
			let ctx = character(23, 5, 45);
			assert_eq!(eval.evaluate(&ctx), 45);
		}
	}
}
