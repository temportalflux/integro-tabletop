use crate::{
	kdl_ext::{AsKdl, DocumentExt, DocumentQueryExt, FromKDL, NodeBuilder, ValueExt},
	system::dnd5e::{
		data::{character::Character, Rest},
		Value,
	},
	utility::{IdPath, Selector},
	GeneralError,
};
use std::{
	path::{Path, PathBuf},
	str::FromStr,
};

#[derive(Clone, PartialEq, Debug)]
pub enum LimitedUses {
	/// This is the most common format.
	/// Usages define a max quantity and a rest that they reset on.
	/// They can be referred to via `Consumer` by the path to the owner (often feature/action).
	Usage(UseCounterData),
	Consumer {
		/// The path to the `LimitedUses::Usage` which this usage consumes from.
		resource: PathBuf,
		/// The amount of uses of `resource` that this usage consumes.
		cost: u32,
	},
}

#[derive(Clone, PartialEq, Debug)]
pub struct UseCounterData {
	/// The number of uses the feature has until it resets.
	pub(crate) max_uses: Value<i32>,
	/// Consumed uses resets when the user takes at least this rest
	/// (a reset on a short rest will also reset on long rest).
	pub(crate) reset_on: Option<Rest>,
	pub(crate) uses_count: Selector<u32>,
}
impl Default for UseCounterData {
	fn default() -> Self {
		Self {
			max_uses: Value::Fixed(0),
			reset_on: None,
			uses_count: Selector::Any {
				id: IdPath::from("uses_consumed"),
				cannot_match: vec![],
			},
		}
	}
}

impl LimitedUses {
	pub fn set_data_path(&self, parent: &std::path::Path) {
		if let Self::Usage(data) = self {
			data.uses_count.set_data_path(parent);
		}
	}

	fn get_use_data<'a>(&'a self, character: &'a Character) -> Option<&'a UseCounterData> {
		match self {
			Self::Usage(data) => Some(data),
			Self::Consumer { resource, .. } => character.features().get_usage(resource),
		}
	}

	pub fn get_uses_path(&self, character: &Character) -> Option<PathBuf> {
		let Some(data) = self.get_use_data(character) else { return None; };
		data.get_data_path()
	}

	pub fn get_uses_consumed(&self, character: &Character) -> u32 {
		let Some(data) = self.get_use_data(character) else { return 0; };
		data.get_uses_consumed(character)
	}

	pub fn get_max_uses(&self, character: &Character) -> i32 {
		let Some(data) = self.get_use_data(character) else { return 0; };
		data.get_max_uses(character)
	}

	pub fn get_reset_rest(&self, character: &Character) -> Option<Rest> {
		let Some(data) = self.get_use_data(character) else { return None; };
		data.get_reset_rest(character)
	}

	pub fn as_consumer(&self) -> Option<(u32, &Path)> {
		match self {
			Self::Usage { .. } => None,
			Self::Consumer { cost, resource } => Some((*cost, resource.as_path())),
		}
	}
}

impl UseCounterData {
	fn get_data_path(&self) -> Option<PathBuf> {
		self.uses_count.get_data_path()
	}

	fn get_uses_consumed(&self, character: &Character) -> u32 {
		character
			.get_selector_value(&self.uses_count)
			.unwrap_or_default()
	}

	fn get_max_uses(&self, character: &Character) -> i32 {
		self.max_uses.evaluate(character)
	}

	fn get_reset_rest(&self, _character: &Character) -> Option<Rest> {
		self.reset_on.clone()
	}
}

impl FromKDL for LimitedUses {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		if let Some(mut node_max_uses) = node.query_opt("scope() > max_uses")? {
			let entry = node_max_uses.next_req()?;
			let max_uses = {
				Value::from_kdl(&mut node_max_uses, entry, |value| {
					Ok(value.as_i64_req()? as i32)
				})?
			};
			let reset_on = match node.query_str_opt("scope() > reset_on", 0)? {
				None => None,
				Some(str) => Some(Rest::from_str(str)?),
			};
			return Ok(Self::Usage(UseCounterData {
				max_uses,
				reset_on,
				..Default::default()
			}));
		}

		if let Some(mut node_resource) = node.query_opt("scope() > resource")? {
			let resource = {
				let path_str = node_resource.next_str_req()?;
				PathBuf::from(path_str)
			};
			let cost = match node.query_opt("scope() > cost")? {
				None => 1,
				Some(mut node) => node.next_i64_req()? as u32,
			};
			return Ok(Self::Consumer { resource, cost });
		}

		return Err(GeneralError(
			"Invalid limited uses, expected a max_uses or resource property.".into(),
		)
		.into());
	}
}

impl AsKdl for LimitedUses {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Usage(use_counter) => {
				node.push_child_t("max_uses", &use_counter.max_uses);
				if let Some(reset_on) = &use_counter.reset_on {
					node.push_child_entry("reset_on", reset_on.to_string());
				}
				node
			}
			Self::Consumer { resource, cost } => {
				node.push_child_entry("resource", resource.display().to_string());
				if *cost > 1 {
					node.push_child_t("cost", cost);
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
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				core::NodeRegistry,
				dnd5e::{data::Rest, evaluator::GetLevelInt},
			},
		};

		static NODE_NAME: &str = "limited_uses";

		fn node_ctx() -> NodeContext {
			NodeContext::registry(NodeRegistry::default_with_eval::<GetLevelInt>())
		}

		#[test]
		fn fixed_uses_permanent() -> anyhow::Result<()> {
			let doc = "
				|limited_uses {
				|    max_uses 2
				|}
			";
			let data = LimitedUses::Usage(UseCounterData {
				max_uses: Value::Fixed(2),
				..Default::default()
			});
			assert_eq_fromkdl!(LimitedUses, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn fixed_uses_reset() -> anyhow::Result<()> {
			let doc = "
				|limited_uses {
				|    max_uses 2
				|    reset_on \"Short\"
				|}
			";
			let data = LimitedUses::Usage(UseCounterData {
				max_uses: Value::Fixed(2),
				reset_on: Some(Rest::Short),
				..Default::default()
			});
			assert_eq_fromkdl!(LimitedUses, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn scaling_uses_reset() -> anyhow::Result<()> {
			let doc = "
				|limited_uses {
				|    max_uses (Evaluator)\"get_level\" class=\"SpecificClass\" {
				|        level 2 1
				|        level 5 2
				|        level 10 4
				|        level 14 5
				|        level 20 -1
				|    }
				|    reset_on \"Long\"
				|}
			";
			let data = LimitedUses::Usage(UseCounterData {
				max_uses: Value::Evaluated(
					GetLevelInt {
						class_name: Some("SpecificClass".into()),
						order_map: [(2, 1), (5, 2), (10, 4), (14, 5), (20, -1)].into(),
					}
					.into(),
				),
				reset_on: Some(Rest::Long),
				..Default::default()
			});
			assert_eq_fromkdl!(LimitedUses, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn resource_simple() -> anyhow::Result<()> {
			let doc = "
				|limited_uses {
				|    resource \"Cleric/level02/Channel Divinity\"
				|}
			";
			let data = LimitedUses::Consumer {
				resource: "Cleric/level02/Channel Divinity".into(),
				cost: 1,
			};
			assert_eq_fromkdl!(LimitedUses, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn resource_with_cost() -> anyhow::Result<()> {
			let doc = "
				|limited_uses {
				|    resource \"Cleric/level02/Channel Divinity\"
				|    cost 4
				|}
			";
			let data = LimitedUses::Consumer {
				resource: "Cleric/level02/Channel Divinity".into(),
				cost: 4,
			};
			assert_eq_fromkdl!(LimitedUses, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
