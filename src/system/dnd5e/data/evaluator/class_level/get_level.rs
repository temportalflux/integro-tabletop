use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode},
	utility::Evaluator,
};
use std::fmt::Debug;

/// Returns the numerical value of the level for a character.
/// Optionally can return the level for a specific class, if `class_name` is specified.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct GetLevel<T> {
	class_name: Option<String>,
	marker: std::marker::PhantomData<T>,
}
impl<T, S> From<Option<S>> for GetLevel<T>
where
	S: ToString,
	T: Default,
{
	fn from(value: Option<S>) -> Self {
		Self {
			class_name: value.map(|s| s.to_string()),
			marker: std::marker::PhantomData::default(),
		}
	}
}

impl<T> crate::utility::TraitEq for GetLevel<T>
where
	T: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl<T> Evaluator for GetLevel<T>
where
	T: 'static + Copy + Debug + Send + Sync + PartialEq,
	usize: num_traits::AsPrimitive<T>,
{
	type Context = Character;
	type Item = T;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		use num_traits::AsPrimitive;
		let class_name = self.class_name.as_ref().map(String::as_str);
		let value = state.level(class_name).as_();
		value
	}
}

impl<T> KDLNode for GetLevel<T> {
	fn id() -> &'static str {
		"get_level"
	}
}

impl<T> FromKDL<DnD5e> for GetLevel<T> {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_system: &DnD5e,
	) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt(value_idx.next())?.map(ToString::to_string);
		Ok(Self {
			class_name,
			marker: Default::default(),
		})
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
		T: 'static + Copy + Debug + Send + Sync + PartialEq,
		usize: num_traits::AsPrimitive<T>,
	{
		DnD5e::defaulteval_parse_kdl::<GetLevel<T>>(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn character_level() -> anyhow::Result<()> {
			let doc = "evaluator \"get_level\"";
			let expected = GetLevel::<u32>::default();
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn class_level() -> anyhow::Result<()> {
			let doc = "evaluator \"get_level\" \"Wizard\"";
			let expected = GetLevel::<u32>::from(Some("Wizard"));
			assert_eq!(from_doc(doc)?, expected.into());
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
		fn character_level_none() {
			let eval = GetLevel::<u32>::default();
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), 0);
		}

		#[test]
		fn character_level_some() {
			let eval = GetLevel::<u32>::default();
			let character = character(&[("SomeClass".into(), 7)]);
			assert_eq!(eval.evaluate(&character), 7);
		}

		#[test]
		fn class_missing() {
			let eval = GetLevel::<u32>::from(Some("MissingClass"));
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), 0);
		}

		#[test]
		fn class_level_some() {
			let eval = GetLevel::<u32>::from(Some("Wizard"));
			let character = character(&[("Wizard".into(), 4), ("Sorcerer".into(), 2)]);
			assert_eq!(eval.evaluate(&character), 4);
		}
	}
}
