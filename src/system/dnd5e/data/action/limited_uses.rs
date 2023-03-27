use crate::{
	kdl_ext::{DocumentExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{character::Character, scaling, Rest},
			FromKDL,
		},
	},
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

	uses_count: Selector<u32>,
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
		_value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let max_uses = {
			let node = node.query_req("scope() > max_uses")?;
			scaling::Value::from_kdl(node, &mut ValueIdx::default(), node_reg)?
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
