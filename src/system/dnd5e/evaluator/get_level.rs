use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt, ValueExt},
	system::dnd5e::data::character::Character,
	utility::Evaluator,
};
use std::{collections::BTreeMap, fmt::Debug};

pub type GetLevelInt = GetLevel<i32>;
pub type GetLevelStr = GetLevel<String>;

/// Returns the numerical value of the level for a character.
/// Optionally can return the level for a specific class, if `class_name` is specified.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct GetLevel<T> {
	pub class_name: Option<String>,
	pub order_map: BTreeMap<usize, T>,
}
impl<T, S> From<Option<S>> for GetLevel<T>
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

crate::impl_trait_eq!(GetLevelInt);
crate::impl_trait_eq!(GetLevelStr);
crate::impl_kdl_node!(GetLevelInt, "get_level");
crate::impl_kdl_node!(GetLevelStr, "get_level_str");

trait GetLevelTy {
	fn from_level(level: usize) -> Self;
	fn from_kdl(entry: &kdl::KdlEntry) -> anyhow::Result<Self>
	where
		Self: Sized;
	fn to_kdl(&self) -> kdl::KdlEntry;
}
impl GetLevelTy for i32 {
	fn from_level(level: usize) -> Self {
		level as i32
	}

	fn from_kdl(entry: &kdl::KdlEntry) -> anyhow::Result<Self> {
		Ok(entry.value().as_i64_req()? as i32)
	}

	fn to_kdl(&self) -> kdl::KdlEntry {
		kdl::KdlEntry::new(*self as i64)
	}
}
impl GetLevelTy for String {
	fn from_level(level: usize) -> Self {
		level.to_string()
	}

	fn from_kdl(entry: &kdl::KdlEntry) -> anyhow::Result<Self> {
		Ok(entry.value().as_str_req()?.to_owned())
	}

	fn to_kdl(&self) -> kdl::KdlEntry {
		kdl::KdlEntry::new(self.clone())
	}
}

impl<T> Evaluator for GetLevel<T>
where
	Self: crate::utility::TraitEq + crate::kdl_ext::KDLNode,
	T: Debug + Send + Sync + Default + GetLevelTy + Clone,
{
	type Context = Character;
	type Item = T;

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
			return T::from_level(level);
		}
		for (key, value) in self.order_map.iter().rev() {
			if level >= *key {
				return value.clone();
			}
		}
		return T::default();
	}
}

impl<T: GetLevelTy> FromKDL for GetLevel<T> {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt("class")?.map(ToString::to_string);
		let mut order_map = BTreeMap::new();
		for node in node.query_all("scope() > level")? {
			let mut ctx = ctx.next_node();
			let level = node.get_i64_req(ctx.consume_idx())? as usize;
			let value = T::from_kdl(node.entry_req(ctx.consume_idx())?)?;
			order_map.insert(level, value);
		}
		Ok(Self {
			class_name,
			order_map,
		})
	}
}

impl<T: GetLevelTy> AsKdl for GetLevel<T> {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(class_name) = &self.class_name {
			node.push_entry(("class", class_name.clone()));
		}
		for (level, value) in &self.order_map {
			node.push_child(
				NodeBuilder::default()
					.with_entry(*level as i64)
					.with_entry(value.to_kdl())
					.build("level"),
			);
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::data::character::Persistent;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::evaluator::test::test_utils};

		test_utils!(GetLevelInt);

		#[test]
		fn character_level() -> anyhow::Result<()> {
			let doc = "evaluator \"get_level\"";
			let data = GetLevelInt::default();
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn class_level() -> anyhow::Result<()> {
			let doc = "evaluator \"get_level\" class=\"Wizard\"";
			let data = GetLevelInt::from(Some("Wizard"));
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
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
			let eval = GetLevelInt::default();
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), 0);
		}

		#[test]
		fn character_level_some() {
			let eval = GetLevelInt::default();
			let character = character(&[("SomeClass".into(), 7)]);
			assert_eq!(eval.evaluate(&character), 7);
		}

		#[test]
		fn class_missing() {
			let eval = GetLevelInt::from(Some("MissingClass"));
			let character = character(&[]);
			assert_eq!(eval.evaluate(&character), 0);
		}

		#[test]
		fn class_level_some() {
			let eval = GetLevelInt::from(Some("Wizard"));
			let character = character(&[("Wizard".into(), 4), ("Sorcerer".into(), 2)]);
			assert_eq!(eval.evaluate(&character), 4);
		}
	}
}
