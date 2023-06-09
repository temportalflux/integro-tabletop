use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::character::Character,
	utility::Evaluator,
};
use std::{collections::BTreeMap, fmt::Debug};

/// Returns the numerical value of the level for a character.
/// Optionally can return the level for a specific class, if `class_name` is specified.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct GetLevel {
	pub class_name: Option<String>,
	pub order_map: BTreeMap<usize, i32>,
}
impl<S> From<Option<S>> for GetLevel
where
	S: ToString,
{
	fn from(value: Option<S>) -> Self {
		Self {
			class_name: value.map(|s| s.to_string()),
			order_map: BTreeMap::default(),
		}
	}
}

crate::impl_trait_eq!(GetLevel);
impl Evaluator for GetLevel {
	type Context = Character;
	type Item = i32;

	fn description(&self) -> Option<String> {
		Some(format!(
			"your {} level",
			match &self.class_name {
				None => "character",
				Some(class_name) => class_name.as_str(),
			}
		))
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let class_name = self.class_name.as_ref().map(String::as_str);
		let level = state.level(class_name);
		if self.order_map.is_empty() {
			return level as i32;
		}
		for (key, value) in self.order_map.iter().rev() {
			if level >= *key {
				return *value;
			}
		}
		return 0;
	}
}

crate::impl_kdl_node!(GetLevel, "get_level");

impl FromKDL for GetLevel {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt("class")?.map(ToString::to_string);
		let mut order_map = BTreeMap::new();
		for node in node.query_all("scope() > level")? {
			let mut ctx = ctx.next_node();
			let level = node.get_i64_req(ctx.consume_idx())? as usize;
			let value = node.get_i64_req(ctx.consume_idx())? as i32;
			order_map.insert(level, value);
		}
		Ok(Self {
			class_name,
			order_map,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		system::{core::NodeRegistry, dnd5e::data::character::Persistent},
		utility::GenericEvaluator,
	};

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
			let doc = "evaluator \"get_level\" class=\"Wizard\"";
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
					current_level: *level,
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
