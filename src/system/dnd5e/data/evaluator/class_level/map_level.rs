use super::GetLevel;
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Evaluator,
};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ByLevel<T> {
	pub class_name: Option<String>,
	pub map: BTreeMap<usize, T>,
}

impl<T, const N: usize> From<[(usize, T); N]> for ByLevel<T> {
	fn from(value: [(usize, T); N]) -> Self {
		Self {
			class_name: None,
			map: BTreeMap::from(value),
		}
	}
}

impl<T> crate::utility::TraitEq for ByLevel<T>
where
	T: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl<T> Evaluator for ByLevel<T>
where
	T: 'static + Clone + Default + Send + Sync + Debug + PartialEq,
{
	type Context = Character;
	type Item = T;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let character_level = GetLevel::from(self.class_name.clone()).evaluate(state);
		for (min_level, value) in self.map.iter().rev() {
			if *min_level <= character_level {
				return value.clone();
			}
		}
		T::default()
	}
}

impl<T> KDLNode for ByLevel<T> {
	fn id() -> &'static str {
		"by_level"
	}
}

impl<T> FromKDL<DnD5e> for ByLevel<T>
where
	T: FromKDL<DnD5e>,
{
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
				let value = T::from_kdl(node, &mut value_idx, system)?;
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
		T: 'static + Clone + Default + Send + Sync + Debug + PartialEq + FromKDL<DnD5e>,
	{
		DnD5e::defaulteval_parse_kdl::<ByLevel<T>>(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn empty() -> anyhow::Result<()> {
			let doc = "evaluator \"by_level\"";
			assert_eq!(from_doc(doc)?, ByLevel::<usize>::default().into());
			Ok(())
		}

		#[test]
		fn class_only() -> anyhow::Result<()> {
			let doc = "evaluator \"by_level\" class=\"SomeClass\"";
			assert_eq!(
				from_doc(doc)?,
				ByLevel::<usize> {
					class_name: Some("SomeClass".into()),
					..Default::default()
				}
				.into()
			);
			Ok(())
		}

		#[test]
		fn character_level() -> anyhow::Result<()> {
			let doc = "evaluator \"by_level\" {
				map 1 2
				map 5 3
				map 9 4
				map 13 5
				map 17 6
			}";
			assert_eq!(
				from_doc(doc)?,
				ByLevel::<usize> {
					map: [(1, 2), (5, 3), (9, 4), (13, 5), (17, 6),].into(),
					..Default::default()
				}
				.into()
			);
			Ok(())
		}

		#[test]
		fn class_level() -> anyhow::Result<()> {
			let doc = "evaluator \"by_level\" class=\"SomeClass\" {
				map 1 2
				map 5 3
				map 9 4
				map 13 5
				map 17 6
			}";
			assert_eq!(
				from_doc(doc)?,
				ByLevel::<usize> {
					class_name: Some("SomeClass".into()),
					map: [(1, 2), (5, 3), (9, 4), (13, 5), (17, 6),].into(),
				}
				.into()
			);
			Ok(())
		}

		#[test]
		fn optional_value() -> anyhow::Result<()> {
			let doc = "evaluator \"by_level\" {
				map 3 \"Electric\"
				map 5
				map 8 \"Boogaloo\"
			}";
			assert_eq!(
				from_doc(doc)?,
				ByLevel::<Option<String>> {
					map: [
						(3, Some("Electric".into())),
						(5, None),
						(8, Some("Boogaloo".into()))
					]
					.into(),
					..Default::default()
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
			let eval = ByLevel::<Option<u32>>::default();
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), None);
		}

		#[test]
		fn character_thresholds() {
			let eval = ByLevel {
				class_name: None,
				map: [
					(1, Some(0)),
					(2, Some(1)),
					(5, Some(2)),
					(7, Some(3)),
					(9, None),
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
			assert_eq!(eval.evaluate(&level0), None);
			assert_eq!(eval.evaluate(&level1), Some(0));
			assert_eq!(eval.evaluate(&level2), Some(1));
			assert_eq!(eval.evaluate(&level3), Some(1));
			assert_eq!(eval.evaluate(&level4), Some(1));
			assert_eq!(eval.evaluate(&level5), Some(2));
			assert_eq!(eval.evaluate(&level6), Some(2));
			assert_eq!(eval.evaluate(&level7), Some(3));
			assert_eq!(eval.evaluate(&level8), Some(3));
			assert_eq!(eval.evaluate(&level9), None);
		}

		#[test]
		fn class_thresholds() {
			let eval = ByLevel {
				class_name: Some("SomeClass".into()),
				map: [
					(1, "A".to_string()),
					(4, "B".to_string()),
					(7, "C".to_string()),
				]
				.into(),
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
			assert_eq!(eval.evaluate(&level0), "");
			assert_eq!(eval.evaluate(&level1), "A");
			assert_eq!(eval.evaluate(&level2), "A");
			assert_eq!(eval.evaluate(&level3), "A");
			assert_eq!(eval.evaluate(&level4), "B");
			assert_eq!(eval.evaluate(&level5), "B");
			assert_eq!(eval.evaluate(&level6), "B");
			assert_eq!(eval.evaluate(&level7), "C");
			assert_eq!(eval.evaluate(&level8), "C");
			assert_eq!(eval.evaluate(&level9), "C");
		}
	}
}
