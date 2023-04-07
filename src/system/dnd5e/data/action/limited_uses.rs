use crate::{
	kdl_ext::{DocumentExt, FromKDL},
	system::dnd5e::data::{character::Character, scaling, Rest},
	utility::{IdPath, Selector},
};
use std::{path::PathBuf, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct LimitedUses {
	/// The number of uses the feature has until it resets.
	pub max_uses: scaling::Value<u32>,
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
			let node = node.query_req("scope() > max_uses")?;
			scaling::Value::from_kdl(node, &mut ctx.next_node())?
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
			system::dnd5e::data::{scaling, Rest},
		};

		fn from_doc(doc: &str) -> anyhow::Result<LimitedUses> {
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > limited_uses")?
				.expect("missing limited_uses node");
			LimitedUses::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn fixed_uses_permanent() -> anyhow::Result<()> {
			let doc = "limited_uses {
				max_uses 2
			}";
			let expected = LimitedUses {
				max_uses: scaling::Value::Fixed(2),
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
				max_uses: scaling::Value::Fixed(2),
				reset_on: Some(Rest::Short),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn scaling_uses_reset() -> anyhow::Result<()> {
			let doc = "limited_uses {
				max_uses (Scaled)\"Level\" class=\"SpecificClass\" {
					level 2 1
					level 5 2
					level 10 4
					level 14 5
					level 20
				}
				reset_on \"Long\"
			}";
			let expected = LimitedUses {
				max_uses: scaling::Value::Scaled(scaling::Basis::Level {
					class_name: Some("SpecificClass".into()),
					level_map: [
						(2, Some(1)),
						(5, Some(2)),
						(10, Some(4)),
						(14, Some(5)),
						(20, None),
					]
					.into(),
				}),
				reset_on: Some(Rest::Long),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
