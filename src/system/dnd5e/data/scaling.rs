mod value;
pub use value::*;
mod basis;
pub use basis::*;

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::dnd5e::{
				data::roll::{Die, Roll},
				FromKDL,
			},
		};

		fn from_doc<T>(doc: &str) -> anyhow::Result<Value<T>>
		where
			T: Clone + DefaultLevelMap + FromKDL,
		{
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > scaling")?
				.expect("missing scaling node");
			Value::<T>::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn fixed_int() -> anyhow::Result<()> {
			let doc = "scaling 1";
			assert_eq!(from_doc(doc)?, Value::Fixed(1u32));
			Ok(())
		}

		#[test]
		fn fixed_roll() -> anyhow::Result<()> {
			let doc = "scaling \"2d8\"";
			assert_eq!(from_doc(doc)?, Value::<Roll>::Fixed((2, Die::D8).into()));
			Ok(())
		}

		#[test]
		fn scaling_level_int_noclass_nomap() -> anyhow::Result<()> {
			let doc = "scaling (Scaled)\"Level\"";
			assert_eq!(
				from_doc(doc)?,
				Value::<u32>::Scaled(Basis::Level {
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
				Value::<u32>::Scaled(Basis::Level {
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
				Value::<u32>::Scaled(Basis::Level {
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
				Value::<Roll>::Scaled(Basis::Level {
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
		use std::collections::BTreeMap;

		use super::*;
		use crate::system::dnd5e::data::{
			character::{Character, Persistent},
			Class, Level,
		};

		fn character(levels: &[(&'static str, usize)]) -> Character {
			let mut persistent = Persistent::default();
			for (name, level) in levels {
				persistent.classes.push(Class {
					name: (*name).to_owned(),
					current_level: *level,
					levels: (0..*level).into_iter().map(|_| Level::default()).collect(),
					..Default::default()
				});
			}
			Character::from(persistent)
		}

		#[test]
		fn fixed() {
			let scaling = Value::<u32>::Fixed(5);
			assert_eq!(scaling.evaluate(&character(&[])), Some(5));
		}

		#[test]
		fn scaling_level_char_nomap() {
			let scaling = Value::<u32>::Scaled(Basis::Level {
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
			let scaling = Value::<u32>::Scaled(Basis::Level {
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
			let scaling = Value::<u32>::Scaled(Basis::Level {
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
			let scaling = Value::<u32>::Scaled(Basis::Level {
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
