use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::character::Character, FromKDL, KDLNode},
	},
	utility::Evaluator,
};
use std::fmt::Debug;

/// Returns the numerical value of the level for a character.
/// Optionally can return the level for a specific class, if `class_name` is specified.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct GetLevel(pub Option<String>);
impl<S> From<Option<S>> for GetLevel
where
	S: ToString,
{
	fn from(value: Option<S>) -> Self {
		Self(value.map(|s| s.to_string()))
	}
}

impl crate::utility::TraitEq for GetLevel {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for GetLevel {
	type Context = Character;
	type Item = i32;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let class_name = self.0.as_ref().map(String::as_str);
		state.level(class_name) as i32
	}
}

impl KDLNode for GetLevel {
	fn id() -> &'static str {
		"get_level"
	}
}

impl FromKDL for GetLevel {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt(value_idx.next())?.map(ToString::to_string);
		Ok(Self(class_name))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{system::dnd5e::data::character::Persistent, utility::GenericEvaluator};

	fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, i32>> {
		NodeRegistry::defaulteval_parse_kdl::<GetLevel>(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn character_level() -> anyhow::Result<()> {
			let doc = "evaluator \"get_level\"";
			let expected = GetLevel::default();
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn class_level() -> anyhow::Result<()> {
			let doc = "evaluator \"get_level\" \"Wizard\"";
			let expected = GetLevel::from(Some("Wizard"));
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
			let eval = GetLevel::default();
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), 0);
		}

		#[test]
		fn character_level_some() {
			let eval = GetLevel::default();
			let character = character(&[("SomeClass".into(), 7)]);
			assert_eq!(eval.evaluate(&character), 7);
		}

		#[test]
		fn class_missing() {
			let eval = GetLevel::from(Some("MissingClass"));
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), 0);
		}

		#[test]
		fn class_level_some() {
			let eval = GetLevel::from(Some("Wizard"));
			let character = character(&[("Wizard".into(), 4), ("Sorcerer".into(), 2)]);
			assert_eq!(eval.evaluate(&character), 4);
		}
	}
}
