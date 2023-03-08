use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Evaluator,
};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ByLevel {
	pub class_name: Option<String>,
	pub map: BTreeMap<usize, kdl::KdlValue>,
}

impl<T, const N: usize> From<[(usize, T); N]> for ByLevel
where
	kdl::KdlValue: From<T>,
{
	fn from(value: [(usize, T); N]) -> Self {
		Self {
			class_name: None,
			map: value
				.into_iter()
				.map(|(lvl, v)| (lvl, kdl::KdlValue::from(v)))
				.collect(),
		}
	}
}

impl crate::utility::TraitEq for ByLevel {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for ByLevel {
	type Context = Character;
	type Item = kdl::KdlValue;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let character_level = state.level(self.class_name.as_ref().map(String::as_str));
		for (min_level, value) in self.map.iter().rev() {
			if *min_level <= character_level {
				return value.clone();
			}
		}
		kdl::KdlValue::Null
	}
}

impl KDLNode for ByLevel {
	fn id() -> &'static str {
		"map_level"
	}
}

impl FromKDL<DnD5e> for ByLevel {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt("class")?.map(ToString::to_string);
		let mut map = BTreeMap::default();
		if let Some(children) = node.children() {
			for node in children.query_all("map")? {
				let mut value_idx = ValueIdx::default();
				let level = node.get_i64(value_idx.next())? as usize;
				let value = node
					.get(value_idx.next())
					.cloned()
					.unwrap_or(kdl::KdlValue::Null);
				map.insert(level, value);
			}
		}
		Ok(Self { class_name, map })
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		system::dnd5e::{data::character::Persistent, DnD5e},
		utility::GenericEvaluator,
	};

	fn from_doc<T>(doc: &str) -> anyhow::Result<GenericEvaluator<Character, T>>
	where
		T: 'static,
	{
		let mut system = DnD5e::default();
		system.register_evaluator::<ByLevel>();
		system.parse_kdl_evaluator::<T>(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn empty() -> anyhow::Result<()> {
			let doc = "evaluator \"map_level\"";
			assert_eq!(from_doc(doc)?, ByLevel::default().into());
			Ok(())
		}

		#[test]
		fn class_only() -> anyhow::Result<()> {
			let doc = "evaluator \"map_level\" class=\"SomeClass\"";
			assert_eq!(
				from_doc(doc)?,
				ByLevel {
					class_name: Some("SomeClass".into()),
					..Default::default()
				}
				.into()
			);
			Ok(())
		}

		#[test]
		fn character_level() -> anyhow::Result<()> {
			let doc = "evaluator \"map_level\" {
				map 1 \"2\"
				map 5 \"3\"
				map 9 \"4\"
				map 13 \"5\"
				map 17 \"6\"
			}";
			assert_eq!(
				from_doc(doc)?,
				ByLevel::from([(1, "2"), (5, "3"), (9, "4"), (13, "5"), (17, "6")]).into()
			);
			Ok(())
		}

		#[test]
		fn class_level() -> anyhow::Result<()> {
			let doc = "evaluator \"map_level\" class=\"SomeClass\" {
				map 1 \"2\"
				map 5 \"3\"
				map 9 \"4\"
				map 13 \"5\"
				map 17 \"6\"
			}";
			assert_eq!(
				from_doc(doc)?,
				ByLevel {
					class_name: Some("SomeClass".into()),
					map: [
						(1, "2".into()),
						(5, "3".into()),
						(9, "4".into()),
						(13, "5".into()),
						(17, "6".into()),
					]
					.into(),
				}
				.into()
			);
			Ok(())
		}
	}

	mod evaluate {
		use super::*;
		use crate::system::dnd5e::data::{Class, Level};

		fn character(levels: &[(String, usize)]) -> Character {
			let mut persistent = Persistent::default();
			for (class_name, level) in levels {
				persistent.classes.push(Class {
					name: class_name.clone(),
					levels: (0..*level).into_iter().map(|_| Level::default()).collect(),
					..Default::default()
				});
			}
			Character::from(persistent)
		}

		#[test]
		fn empty() {
			let eval = ByLevel::default();
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), kdl::KdlValue::Null);
		}

		#[test]
		fn character_thresholds() {
			let eval = ByLevel {
				class_name: None,
				map: [
					(1, Some(0).into()),
					(2, Some(1).into()),
					(5, Some(2).into()),
					(7, Some(3).into()),
					(9, None::<i64>.into()),
				]
				.into(),
			};
			let level0 = character(&[]);
			let level1 = character(&[("SomeClass".into(), 1)]);
			let level2 = character(&[("SomeClass".into(), 1), ("OtherClass".into(), 1)]);
			let level3 = character(&[("SomeClass".into(), 2), ("OtherClass".into(), 1)]);
			let level4 = character(&[("SomeClass".into(), 2), ("OtherClass".into(), 2)]);
			let level5 = character(&[("SomeClass".into(), 2), ("OtherClass".into(), 3)]);
			let level6 = character(&[("SomeClass".into(), 2), ("OtherClass".into(), 4)]);
			let level7 = character(&[("SomeClass".into(), 3), ("OtherClass".into(), 4)]);
			let level8 = character(&[("SomeClass".into(), 4), ("OtherClass".into(), 4)]);
			let level9 = character(&[("SomeClass".into(), 4), ("OtherClass".into(), 5)]);
			assert_eq!(eval.evaluate(&level0), kdl::KdlValue::Null);
			assert_eq!(eval.evaluate(&level1), 0.into());
			assert_eq!(eval.evaluate(&level2), 1.into());
			assert_eq!(eval.evaluate(&level3), 1.into());
			assert_eq!(eval.evaluate(&level4), 1.into());
			assert_eq!(eval.evaluate(&level5), 2.into());
			assert_eq!(eval.evaluate(&level6), 2.into());
			assert_eq!(eval.evaluate(&level7), 3.into());
			assert_eq!(eval.evaluate(&level8), 3.into());
			assert_eq!(eval.evaluate(&level9), kdl::KdlValue::Null);
		}

		#[test]
		fn class_thresholds() {
			let eval = ByLevel {
				class_name: Some("SomeClass".into()),
				map: [(1, "A".into()), (4, "B".into()), (7, "C".into())].into(),
			};
			let level0 = character(&[]);
			let level1 = character(&[("SomeClass".into(), 1), ("OtherClass".into(), 3)]);
			let level2 = character(&[("SomeClass".into(), 2), ("OtherClass".into(), 3)]);
			let level3 = character(&[("SomeClass".into(), 3), ("OtherClass".into(), 3)]);
			let level4 = character(&[("SomeClass".into(), 4), ("OtherClass".into(), 3)]);
			let level5 = character(&[("SomeClass".into(), 5), ("OtherClass".into(), 3)]);
			let level6 = character(&[("SomeClass".into(), 6), ("OtherClass".into(), 3)]);
			let level7 = character(&[("SomeClass".into(), 7), ("OtherClass".into(), 3)]);
			let level8 = character(&[("SomeClass".into(), 8), ("OtherClass".into(), 3)]);
			let level9 = character(&[("SomeClass".into(), 9), ("OtherClass".into(), 3)]);
			assert_eq!(eval.evaluate(&level0), kdl::KdlValue::Null);
			assert_eq!(eval.evaluate(&level1), "A".into());
			assert_eq!(eval.evaluate(&level2), "A".into());
			assert_eq!(eval.evaluate(&level3), "A".into());
			assert_eq!(eval.evaluate(&level4), "B".into());
			assert_eq!(eval.evaluate(&level5), "B".into());
			assert_eq!(eval.evaluate(&level6), "B".into());
			assert_eq!(eval.evaluate(&level7), "C".into());
			assert_eq!(eval.evaluate(&level8), "C".into());
			assert_eq!(eval.evaluate(&level9), "C".into());
		}
	}
}
