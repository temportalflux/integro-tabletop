use crate::{
	kdl_ext::NodeExt,
	system::dnd5e::{data::character::Character, FromKDL},
	utility::NotInList,
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
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Level" => {
				let class_name = node.get_str_opt("class")?.map(str::to_owned);
				let mut level_map = BTreeMap::new();
				for node in node.query_all("scope() > level")? {
					let mut ctx = ctx.next_node();
					let threshold = node.get_i64_req(ctx.consume_idx())? as usize;
					let value = match node.get(ctx.peak_idx()).is_some() {
						false => None,
						true => Some(T::from_kdl(node, &mut ctx)?),
					};
					level_map.insert(threshold, value);
				}
				Ok(Self::Level {
					class_name,
					level_map,
				})
			}
			name => Err(NotInList(name.into(), vec!["Level"]).into()),
		}
	}
}
