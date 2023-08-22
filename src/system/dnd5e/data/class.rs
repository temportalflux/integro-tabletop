use super::{character::Character, roll::Die};
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::{mutator::AddMaxHitPoints, BoxedMutator, SystemComponent, Value},
	},
	utility::{selector, MutatorGroup},
};
use std::{path::Path, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct Class {
	pub id: SourceId,
	pub name: String,
	pub description: String,
	pub hit_die: Die,
	pub hit_die_selector: selector::Value<Character, u32>,
	pub current_level: usize,
	/// Mutators that are applied only when this class is the primary class (not multiclassing).
	pub mutators: Vec<BoxedMutator>,
	pub levels: Vec<Level>,
	pub subclass_selection_level: Option<usize>,
	pub subclass: Option<Subclass>,
	// TODO: `multiclass-req` data node (already in data, just not in structures yet)
}

impl Default for Class {
	fn default() -> Self {
		Self {
			id: Default::default(),
			name: Default::default(),
			description: Default::default(),
			hit_die: Default::default(),
			hit_die_selector: selector::Value::Options(selector::ValueOptions {
				id: "hit_die".into(),
				..Default::default()
			}),
			current_level: Default::default(),
			mutators: Default::default(),
			levels: Default::default(),
			subclass_selection_level: Default::default(),
			subclass: Default::default(),
		}
	}
}

impl Class {
	pub fn iter_levels<'a>(&'a self, all: bool) -> impl Iterator<Item = LevelWithIndex<'a>> + 'a {
		self.levels
			.iter()
			.enumerate()
			.filter(move |(idx, _)| all || *idx < self.current_level)
			.map(|(idx, lvl)| LevelWithIndex(idx, lvl))
	}
}

impl MutatorGroup for Class {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_self = parent.join(&self.name);
		self.hit_die_selector.set_data_path(&path_to_self);
		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
		for level in self.iter_levels(true) {
			level.set_data_path(&path_to_self);
		}
		if let Some(subclass) = &self.subclass {
			subclass.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
		for level in self.iter_levels(false) {
			stats.apply_from(&level, &path_to_self);
		}
		if let Some(subclass) = &self.subclass {
			stats.apply_from(subclass, &path_to_self);
		}
	}
}

impl SystemComponent for Class {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name.clone(),
		})
	}
}

crate::impl_kdl_node!(Class, "class");

impl FromKDL for Class {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.query_source_req()?;

		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let hit_die = Die::from_str(node.query_str_req("scope() > hit-die", 0)?)?;
		let current_level = node.get_i64_opt("level")?.unwrap_or_default() as usize;

		let mutators = node.query_all_t("scope() > mutator")?;

		let subclass_selection_level = node
			.query_i64_opt("scope() > subclass-level", 0)?
			.map(|v| v as usize);
		let subclass = node.query_opt_t::<Subclass>("scope() > subclass")?;

		let mut levels = Vec::with_capacity(20);
		levels.resize_with(20, Default::default);
		for mut node in &mut node.query_all("scope() > level")? {
			let order = node.next_i64_req()? as usize;
			let idx = order - 1;
			levels[idx] = Level::from_kdl(&mut node)?;
		}

		Ok(Self {
			id,
			name,
			description,
			hit_die,
			current_level,
			mutators,
			levels,
			subclass_selection_level,
			subclass,
			..Default::default()
		})
	}
}
// TODO AsKdl: from/as tests for Class, Level, Subclass
impl AsKdl for Class {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("name", self.name.clone()));
		if self.current_level != 0 {
			node.push_entry(("level", self.current_level as i64));
		}

		node.push_child_opt_t("source", &self.id);
		node.push_child_opt_t("description", &self.description);
		node.push_child_entry("hit-die", self.hit_die.to_string());
		if let Some(level) = &self.subclass_selection_level {
			node.push_child_t("subclass-level", level);
		}

		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
		}

		for (idx, level) in self.levels.iter().enumerate() {
			let level_node = level.as_kdl();
			if level_node.is_empty() {
				continue;
			}
			node.push_child(
				NodeBuilder::default()
					.with_entry((idx + 1) as i64)
					.with_extension(level_node)
					.build("level"),
			);
		}

		if let Some(subclass) = &self.subclass {
			node.push_child_t("subclass", subclass);
		}

		node
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Level {
	pub hit_points: selector::Value<Character, u32>,
	pub mutators: Vec<BoxedMutator>,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			hit_points: selector::Value::Options(selector::ValueOptions {
				id: "hit_points".into(),
				..Default::default()
			}),
			mutators: Default::default(),
		}
	}
}

impl Level {
	pub fn is_empty(&self) -> bool {
		self.mutators.is_empty()
	}
}

impl FromKDL for Level {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let hit_points = selector::Value::Options(selector::ValueOptions {
			id: "hit_points".into(),
			..Default::default()
		});

		let mutators = node.query_all_t("scope() > mutator")?;

		Ok(Self {
			hit_points,
			mutators,
		})
	}
}
impl AsKdl for Level {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
		}
		node
	}
}

pub struct LevelWithIndex<'a>(usize, &'a Level);
impl<'a> LevelWithIndex<'a> {
	pub fn index(&self) -> usize {
		self.0
	}

	pub fn level(&self) -> &'a Level {
		self.1
	}

	fn level_name(&self) -> String {
		format!("level{:02}", self.0 + 1)
	}
}
impl<'a> MutatorGroup for LevelWithIndex<'a> {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(self.level_name());
		self.1.hit_points.set_data_path(&path_to_self);
		for mutator in &self.1.mutators {
			mutator.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(self.level_name());
		if let Some(hit_points) = stats.resolve_selector(&self.1.hit_points) {
			let mutator = AddMaxHitPoints {
				id: None,
				value: Value::Fixed(hit_points as i32),
			};
			stats.apply(&mutator.into(), &path_to_self);
		}
		for mutator in &self.1.mutators {
			stats.apply(mutator, &path_to_self);
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Subclass {
	pub source_id: SourceId,
	pub class_name: String,
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub levels: Vec<Level>,
}

impl SystemComponent for Subclass {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name.clone(),
		})
	}
}

crate::impl_kdl_node!(Subclass, "subclass");

impl Subclass {
	fn iter_levels<'a>(&'a self) -> impl Iterator<Item = LevelWithIndex<'a>> + 'a {
		self.levels
			.iter()
			.enumerate()
			.map(|(idx, lvl)| LevelWithIndex(idx, lvl))
	}
}

impl MutatorGroup for Subclass {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
		for level in self.iter_levels() {
			level.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
		for level in self.iter_levels() {
			stats.apply_from(&level, &path_to_self);
		}
	}
}

impl FromKDL for Subclass {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let class_name = node.get_str_req("class")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let mutators = node.query_all_t("scope() > mutator")?;
		let mut levels = Vec::with_capacity(20);
		levels.resize_with(20, Default::default);
		for mut node in &mut node.query_all("scope() > level")? {
			let order = node.next_i64_req()? as usize;
			let idx = order - 1;
			levels[idx] = Level::from_kdl(&mut node)?;
		}

		Ok(Self {
			source_id: Default::default(),
			name,
			description,
			class_name,
			mutators,
			levels,
		})
	}
}
impl AsKdl for Subclass {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("class", self.class_name.clone()));
		node.push_entry(("name", self.name.clone()));
		node.push_child_opt_t("source", &self.source_id);
		node.push_child_opt_t("description", &self.description);

		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
		}

		for (idx, level) in self.levels.iter().enumerate() {
			let level_node = level.as_kdl();
			if level_node.is_empty() {
				continue;
			}
			node.push_child(
				NodeBuilder::default()
					.with_entry((idx + 1) as i64)
					.with_extension(level_node)
					.build("level"),
			);
		}

		node
	}
}
