use crate::{kdl_ext::NodeContext, system::dnd5e::data::character::Character, utility::NotInList};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::collections::BTreeMap;

mod level;
pub use level::*;

#[derive(Clone, PartialEq, Debug)]
pub enum Basis<T> {
	Level { class_name: Option<String>, level_map: BTreeMap<usize, Option<T>> },
}

impl<T> Basis<T>
where
	T: Clone + DefaultLevelMap,
{
	pub fn evaluate(&self, character: &Character) -> Option<T> {
		match self {
			Self::Level { class_name, level_map } => {
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

impl<T> FromKdl<NodeContext> for Basis<T>
where
	T: Clone + DefaultLevelMap + FromKdl<NodeContext>,
	anyhow::Error: From<T::Error>,
{
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		match node.next_str_req()? {
			"Level" => {
				let class_name = node.get_str_opt("class")?.map(str::to_owned);
				let mut level_map = BTreeMap::new();
				for mut node in &mut node.query_all("scope() > level")? {
					let threshold = node.next_i64_req()? as usize;
					let value = match node.peak_opt().is_some() {
						false => None,
						true => Some(T::from_kdl(&mut node)?),
					};
					level_map.insert(threshold, value);
				}
				Ok(Self::Level { class_name, level_map })
			}
			name => Err(NotInList(name.into(), vec!["Level"]).into()),
		}
	}
}
impl<T> AsKdl for Basis<T>
where
	T: Clone + DefaultLevelMap + AsKdl,
{
	fn as_kdl(&self) -> NodeBuilder {
		match self {
			Self::Level { class_name, level_map } => {
				let mut node = NodeBuilder::default();
				node.entry("Level");
				if let Some(class_name) = class_name {
					node.entry(("class", class_name.clone()));
				}
				if !level_map.is_empty() {
					for (threshold, value) in level_map {
						node.child({
							let mut node = NodeBuilder::default();
							node.entry(*threshold as i64);
							if let Some(value) = value {
								node += value.as_kdl();
							}
							node.build("level")
						});
					}
				}
				node
			}
		}
	}
}
