use crate::{
	kdl_ext::{DocumentExt, EntryExt, NodeExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{
			data::{character::Character, roll::Roll, Rest},
			FromKDL, Value,
		},
	},
	GeneralError,
};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct LimitedUses {
	/// The number of uses the feature has until it resets.
	/// TODO: Use a ScalingUses instead of Value, which always scale in relation to some evaluator (in most cases, get_level)
	pub max_uses: Value<Option<usize>>,
	/// Consumed uses resets when the user takes at least this rest
	/// (a reset on a short rest will also reset on long rest).
	pub reset_on: Option<Rest>,
}

impl FromKDL for LimitedUses {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let max_uses = {
			// Temporary code, until I can implement scaling uses
			let node = node.query_req("scope() > max_uses")?;
			let max_uses = node.get_i64_req(0)? as usize;
			Value::Fixed(Some(max_uses))
		};
		let reset_on = match node.query_str_opt("scope() > reset_on", 0)? {
			None => None,
			Some(str) => Some(Rest::from_str(str)?),
		};
		Ok(Self { max_uses, reset_on })
	}
}

#[derive(Clone, PartialEq, Debug)]
enum ScalingValue<T> {
	Fixed(T),
	Scaled(ScalingBasis<T>),
}
impl<T> ScalingValue<T>
where
	T: Clone + DefaultLevelMap,
{
	pub fn evaluate(&self, character: &Character) -> Option<T> {
		match self {
			Self::Fixed(value) => Some(value.clone()),
			Self::Scaled(basis) => basis.evaluate(character),
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
enum ScalingBasis<T> {
	Level {
		class_name: Option<String>,
		level_map: BTreeMap<usize, Option<T>>,
	},
}
impl<T> ScalingBasis<T>
where
	T: Clone + DefaultLevelMap,
{
	fn evaluate(&self, character: &Character) -> Option<T> {
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
trait DefaultLevelMap {
	fn default_for_level(level: usize) -> Option<Self>
	where
		Self: Sized;
}
impl DefaultLevelMap for u32 {
	fn default_for_level(level: usize) -> Option<Self> {
		Some(level as u32)
	}
}
impl DefaultLevelMap for Roll {
	fn default_for_level(_level: usize) -> Option<Self> {
		None
	}
}

impl<T> FromKDL for ScalingValue<T>
where
	T: Clone + DefaultLevelMap + FromKDL,
{
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		match node.entry_req(**value_idx)?.type_opt() {
			None => Ok(Self::Fixed(T::from_kdl(node, value_idx, node_reg)?)),
			Some("Scaled") => Ok(Self::Scaled(ScalingBasis::from_kdl(
				node, value_idx, node_reg,
			)?)),
			Some(type_name) => Err(GeneralError(format!(
				"Invalid type name {type_name:?}, expected no type or Scaled."
			))
			.into()),
		}
	}
}
impl<T> FromKDL for ScalingBasis<T>
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

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::{
			core::NodeRegistry,
			dnd5e::data::roll::{Die, Roll},
		};

		fn from_doc<T>(doc: &str) -> anyhow::Result<ScalingValue<T>>
		where
			T: Clone + DefaultLevelMap + FromKDL,
		{
			let node_reg = NodeRegistry::default();
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > scaling")?
				.expect("missing scaling node");
			let mut idx = ValueIdx::default();
			ScalingValue::<T>::from_kdl(node, &mut idx, &node_reg)
		}

		#[test]
		fn fixed_int() -> anyhow::Result<()> {
			let doc = "scaling 1";
			assert_eq!(from_doc(doc)?, ScalingValue::Fixed(1u32));
			Ok(())
		}

		#[test]
		fn fixed_roll() -> anyhow::Result<()> {
			let doc = "scaling \"2d8\"";
			assert_eq!(from_doc(doc)?, ScalingValue::<Roll>::Fixed((2, Die::D8).into()));
			Ok(())
		}

		#[test]
		fn scaling_level_int_noclass_nomap() -> anyhow::Result<()> {
			let doc = "scaling (Scaled)\"Level\"";
			assert_eq!(
				from_doc(doc)?,
				ScalingValue::<u32>::Scaled(ScalingBasis::Level {
					class_name: None,
					level_map: [].into()
				})
			);
			Ok(())
		}

		#[test]
		fn scaling_level_int_nomap() -> anyhow::Result<()> {
			let doc = "scaling (Scaled)\"Level\" class=\"Barbarian\"";
			assert_eq!(
				from_doc(doc)?,
				ScalingValue::<u32>::Scaled(ScalingBasis::Level {
					class_name: Some("Barbarian".into()),
					level_map: [].into()
				})
			);
			Ok(())
		}

		#[test]
		fn scaling_level_int() -> anyhow::Result<()> {
			let doc = "scaling (Scaled)\"Level\" class=\"Barbarian\" {
				level 1 2
				level 4 3
				level 7 4
				level 14 5
				level 18
			}";
			assert_eq!(
				from_doc(doc)?,
				ScalingValue::<u32>::Scaled(ScalingBasis::Level {
					class_name: Some("Barbarian".into()),
					level_map: [
						(1, Some(2)),
						(4, Some(3)),
						(7, Some(4)),
						(14, Some(5)),
						(18, None)
					]
					.into()
				})
			);
			Ok(())
		}

		#[test]
		fn scaling_level_roll_noclass() -> anyhow::Result<()> {
			let doc = "scaling (Scaled)\"Level\" {
				level 1 \"1d8\"
				level 5 \"2d8\"
				level 9 \"3d8\"
				level 16 \"4d8\"
			}";
			assert_eq!(
				from_doc(doc)?,
				ScalingValue::<Roll>::Scaled(ScalingBasis::Level {
					class_name: None,
					level_map: [
						(1, Some((1, Die::D8).into())),
						(5, Some((2, Die::D8).into())),
						(9, Some((3, Die::D8).into())),
						(16, Some((4, Die::D8).into())),
					]
					.into()
				})
			);
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::system::dnd5e::data::{character::Persistent, Class, Level};

		fn character(levels: &[(&'static str, usize)]) -> Character {
			let mut persistent = Persistent::default();
			for (name, level) in levels {
				persistent.classes.push(Class {
					name: (*name).to_owned(),
					levels: (0..*level).into_iter().map(|_| Level::default()).collect(),
					..Default::default()
				});
			}
			Character::from(persistent)
		}

		#[test]
		fn fixed() {
			let scaling = ScalingValue::<u32>::Fixed(5);
			assert_eq!(scaling.evaluate(&character(&[])), Some(5));
		}

		#[test]
		fn scaling_level_char_nomap() {
			let scaling = ScalingValue::<u32>::Scaled(ScalingBasis::Level {
				class_name: None,
				level_map: BTreeMap::new(),
			});
			assert_eq!(scaling.evaluate(&character(&[])), Some(0));
			assert_eq!(scaling.evaluate(&character(&[("Any", 4)])), Some(4));
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 4), ("Another", 5)])),
				Some(9)
			);
		}

		#[test]
		fn scaling_level_char_map() {
			let scaling = ScalingValue::<u32>::Scaled(ScalingBasis::Level {
				class_name: None,
				level_map: [
					(1, Some(4)),
					(3, Some(5)),
					(8, Some(10)),
					(16, Some(14)),
					(19, Some(15)),
					(20, None),
				]
				.into(),
			});
			assert_eq!(scaling.evaluate(&character(&[])), None);
			assert_eq!(scaling.evaluate(&character(&[("Any", 1)])), Some(4));
			assert_eq!(scaling.evaluate(&character(&[("Any", 2)])), Some(4));
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 2), ("Lvl", 1)])),
				Some(5)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 5), ("Lvl", 2)])),
				Some(5)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 5), ("Lvl", 3)])),
				Some(10)
			);
			assert_eq!(scaling.evaluate(&character(&[("Any", 13)])), Some(10));
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 12), ("Lvl", 4)])),
				Some(14)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 11), ("Lvl", 7)])),
				Some(14)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("Any", 10), ("Lvl", 9)])),
				Some(15)
			);
			assert_eq!(scaling.evaluate(&character(&[("Any", 20)])), None);
		}

		#[test]
		fn scaling_level_class_nomap() {
			let scaling = ScalingValue::<u32>::Scaled(ScalingBasis::Level {
				class_name: Some("ClassA".into()),
				level_map: BTreeMap::new(),
			});
			assert_eq!(scaling.evaluate(&character(&[])), Some(0));
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 1)])), Some(1));
			assert_eq!(scaling.evaluate(&character(&[("ClassB", 2)])), Some(0));
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 3), ("ClassB", 2)])),
				Some(3)
			);
		}

		#[test]
		fn scaling_level_class_map() {
			let scaling = ScalingValue::<u32>::Scaled(ScalingBasis::Level {
				class_name: Some("ClassA".into()),
				level_map: [
					(1, Some(1)),
					(3, Some(2)),
					(6, Some(3)),
					(10, Some(5)),
					(15, Some(8)),
					(20, None),
				]
				.into(),
			});
			assert_eq!(scaling.evaluate(&character(&[])), None);
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 1)])), Some(1));
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 2)])), Some(1));
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 3)])), Some(2));
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 5)])), Some(2));
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 6)])), Some(3));
			assert_eq!(scaling.evaluate(&character(&[("ClassA", 7)])), Some(3));
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 9), ("Any", 1)])),
				Some(3)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 10), ("Any", 2)])),
				Some(5)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 14), ("Any", 3)])),
				Some(5)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 15), ("Any", 4)])),
				Some(8)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 19), ("Any", 5)])),
				Some(8)
			);
			assert_eq!(
				scaling.evaluate(&character(&[("ClassA", 20), ("Any", 6)])),
				None
			);
		}
	}
}
