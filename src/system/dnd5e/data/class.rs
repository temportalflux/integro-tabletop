use super::{character::Character, roll::Die, Feature};
use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeContext, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{mutator::AddMaxHitPoints, BoxedMutator, SystemComponent, Value},
	},
	utility::{MutatorGroup, Selector},
};
use std::{path::Path, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Class {
	pub source_id: SourceId,
	pub name: String,
	pub description: String,
	pub hit_die: Die,
	/// Mutators that are applied only when this class is the primary class (not multiclassing).
	pub mutators: Vec<BoxedMutator>,
	pub levels: Vec<Level>,
	pub subclass_selection_level: Option<usize>,
	pub subclass: Option<Subclass>,
}

impl Class {
	pub fn level_count(&self) -> usize {
		self.levels.len()
	}

	fn iter_levels<'a>(&'a self) -> impl Iterator<Item = LevelWithIndex<'a>> + 'a {
		self.levels
			.iter()
			.enumerate()
			.map(|(idx, lvl)| LevelWithIndex(idx, lvl))
	}
}

impl MutatorGroup for Class {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
		for level in self.iter_levels() {
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
		for level in self.iter_levels() {
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
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let hit_die = Die::from_str(node.query_str_req("scope() > hit-die", 0)?)?;

		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let subclass_selection_level = node
			.query_i64_opt("scope() > subclass-level", 0)?
			.map(|v| v as usize);
		let subclass = match node.query_opt("scope() > subclass")? {
			None => None,
			Some(node) => Some(Subclass::from_kdl(node, &mut ctx.next_node())?),
		};

		let mut levels = Vec::with_capacity(20);
		levels.resize_with(20, Default::default);
		for node in node.query_all("scope() > level")? {
			let mut ctx = ctx.next_node();
			let order = node.get_i64_req(ctx.consume_idx())? as usize;
			let idx = order - 1;
			levels[idx] = Level::from_kdl(node, &mut ctx)?;
		}

		Ok(Self {
			source_id: SourceId::default(),
			name,
			description,
			hit_die,
			mutators,
			levels,
			subclass_selection_level,
			subclass,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Level {
	pub hit_points: Selector<u32>,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<Feature>,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			hit_points: Selector::Any {
				id: Some("hit_points").into(),
				cannot_match: Default::default(),
			},
			mutators: Default::default(),
			features: Default::default(),
		}
	}
}

impl Level {
	pub fn is_empty(&self) -> bool {
		self.mutators.is_empty() && self.features.is_empty()
	}
}

impl FromKDL for Level {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let hit_points = Selector::Any {
			id: Some("hit_points").into(),
			cannot_match: Default::default(),
		};

		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let mut features = Vec::new();
		for entry_node in node.query_all("scope() > feature")? {
			features.push(Feature::from_kdl(entry_node, &mut ctx.next_node())?.into());
		}

		Ok(Self {
			hit_points,
			mutators,
			features,
		})
	}
}

struct LevelWithIndex<'a>(usize, &'a Level);
impl<'a> LevelWithIndex<'a> {
	fn level_name(&self) -> String {
		format!("level{:02}", self.0 + 1)
	}
}
impl<'a> MutatorGroup for LevelWithIndex<'a> {
	type Target = Character;

	// TODO: SelectorMeta for `Level::hit_points` integer field

	fn set_data_path(&self, parent: &Path) {
		let path_to_self = parent.join(self.level_name());
		self.1.hit_points.set_data_path(&path_to_self);
		for mutator in &self.1.mutators {
			mutator.set_data_path(&path_to_self);
		}
		for feature in &self.1.features {
			feature.set_data_path(&path_to_self);
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
		for feature in &self.1.features {
			stats.add_feature(feature, &path_to_self);
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Subclass {
	pub source_id: SourceId,
	pub class_name: String,
	pub name: String,
	pub description: String,
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
		for level in self.iter_levels() {
			level.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for level in self.iter_levels() {
			stats.apply_from(&level, &path_to_self);
		}
	}
}

impl FromKDL for Subclass {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let class_name = node.get_str_req("class")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();

		let mut levels = Vec::with_capacity(20);
		levels.resize_with(20, Default::default);
		for node in node.query_all("scope() > level")? {
			let mut ctx = ctx.next_node();
			let order = node.get_i64_req(ctx.consume_idx())? as usize;
			let idx = order - 1;
			levels[idx] = Level::from_kdl(node, &mut ctx)?;
		}

		Ok(Self {
			source_id: Default::default(),
			name,
			description,
			class_name,
			levels,
		})
	}
}
