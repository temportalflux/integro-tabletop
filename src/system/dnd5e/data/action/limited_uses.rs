use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt, ValueExt},
	system::dnd5e::{
		data::{character::Character, Rest},
		Value,
	},
	utility::{IdPath, Selector},
};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct LimitedUses {
	/// The number of uses the feature has until it resets.
	pub max_uses: Value<i32>,
	/// Consumed uses resets when the user takes at least this rest
	/// (a reset on a short rest will also reset on long rest).
	pub reset_on: Option<Rest>,

	pub uses_count: Selector<u32>,
}

impl Default for LimitedUses {
	fn default() -> Self {
		Self {
			max_uses: Default::default(),
			reset_on: Default::default(),
			uses_count: Selector::Any {
				id: IdPath::from("uses_consumed"),
				cannot_match: vec![],
			},
		}
	}
}

impl LimitedUses {
	pub fn set_data_path(&self, parent: &std::path::Path) {
		self.uses_count.set_data_path(parent);
	}

	pub fn get_uses_path(&self) -> PathBuf {
		self.uses_count
			.get_data_path()
			.expect("LimitedUses::uses_count should have a path when interacted with")
	}

	pub fn get_uses_consumed(&self, character: &Character) -> u32 {
		character
			.get_selector_value(&self.uses_count)
			.unwrap_or_default()
	}
}

impl FromKDL for LimitedUses {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let max_uses = {
			let mut ctx = ctx.next_node();
			let node = node.query_req("scope() > max_uses")?;
			Value::from_kdl(
				node,
				node.entry_req(ctx.consume_idx())?,
				&mut ctx,
				|value| Ok(value.as_i64_req()? as i32),
			)?
		};
		let reset_on = match node.query_str_opt("scope() > reset_on", 0)? {
			None => None,
			Some(str) => Some(Rest::from_str(str)?),
		};
		Ok(Self {
			max_uses,
			reset_on,
			..Default::default()
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;
	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{dnd5e::{data::Rest, evaluator::GetLevel}, core::NodeRegistry},
		};

		fn from_doc(doc: &str) -> anyhow::Result<LimitedUses> {
			let mut ctx = NodeContext::registry(NodeRegistry::default_with_eval::<GetLevel>());
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > limited_uses")?
				.expect("missing limited_uses node");
			LimitedUses::from_kdl(node, &mut ctx)
		}

		#[test]
		fn fixed_uses_permanent() -> anyhow::Result<()> {
			let doc = "limited_uses {
				max_uses 2
			}";
			let expected = LimitedUses {
				max_uses: Value::Fixed(2),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn fixed_uses_reset() -> anyhow::Result<()> {
			let doc = "limited_uses {
				max_uses 2
				reset_on \"Short\"
			}";
			let expected = LimitedUses {
				max_uses: Value::Fixed(2),
				reset_on: Some(Rest::Short),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn scaling_uses_reset() -> anyhow::Result<()> {
			let doc = "limited_uses {
				max_uses (Evaluator)\"get_level\" class=\"SpecificClass\" {
					level 2 1
					level 5 2
					level 10 4
					level 14 5
					level 20 -1
				}
				reset_on \"Long\"
			}";
			let expected = LimitedUses {
				max_uses: Value::Evaluated(
					GetLevel {
						class_name: Some("SpecificClass".into()),
						order_map: [(2, 1), (5, 2), (10, 4), (14, 5), (20, -1)].into(),
					}
					.into(),
				),
				reset_on: Some(Rest::Long),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
