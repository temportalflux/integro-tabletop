use crate::{
	kdl_ext::{NodeExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::character::Character, FromKDL},
	},
	GeneralError,
};
use std::collections::BTreeMap;

mod level;
pub use level::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Basis<T> {
	Level {
		class_name: Option<String>,
		level_map: BTreeMap<usize, Option<T>>,
	},
}

impl<T> Basis<T>
where
	T: Clone + DefaultLevelMap,
{
	pub fn evaluate(&self, character: &Character) -> Option<T> {
		match self {
			Self::Level {
				class_name,
				level_map,
			} => {
				let level = character.level(class_name.as_ref().map(String::as_str));
				if level_map.is_empty() {
					return T::default_for_level(level);
				}
				for (min_level, value) in level_map.iter().rev() {
					if *min_level <= level {
						return value.clone();
					}
				}
				None
			}
		}
	}
}

impl<T> FromKDL for Basis<T>
where
	T: Clone + DefaultLevelMap + FromKDL,
{
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		match node.get_str_req(value_idx.next())? {
			"Level" => {
				let class_name = node.get_str_opt("class")?.map(str::to_owned);
				let mut level_map = BTreeMap::new();
				for node in node.query_all("scope() > level")? {
					let mut value_idx = ValueIdx::default();
					let threshold = node.get_i64_req(value_idx.next())? as usize;
					let value = match node.get(*value_idx).is_some() {
						false => None,
						true => Some(T::from_kdl(node, &mut value_idx, node_reg)?),
					};
					level_map.insert(threshold, value);
				}
				Ok(Self::Level {
					class_name,
					level_map,
				})
			}
			name => {
				Err(GeneralError(format!("Invalid scaling name {name:?}, expected Level.")).into())
			}
		}
	}
}
