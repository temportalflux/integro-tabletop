use crate::{
	path_map::PathMap,
	system::{
		core::SourceId,
		dnd5e::{
			data::{
				bundle::{Background, Lineage, Race, RaceVariant, Upbringing},
				character::Character,
				evaluator::{operator::Product, GetAbilityModifier, GetLevel},
				item,
				mutator::AddMaxHitPoints,
				Ability, BoxedFeature, Class, Condition,
			},
			Value,
		},
	},
	utility::MutatorGroup,
};
use enum_map::EnumMap;
use std::{collections::BTreeMap, path::Path, sync::Arc};

mod description;
pub use description::*;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct NamedGroups {
	pub race: Vec<Race>,
	pub race_variant: Vec<RaceVariant>,
	pub lineage: Vec<Lineage>,
	pub upbringing: Vec<Upbringing>,
	pub background: Vec<Background>,
}

/// Core character data which is (de)serializable and
/// from which the derived data can be compiled.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Persistent {
	pub named_groups: NamedGroups,
	pub classes: Vec<Class>,
	pub feats: Vec<BoxedFeature>,
	pub description: Description,
	pub ability_scores: EnumMap<Ability, u32>,
	pub selected_values: PathMap<String>,
	pub inventory: item::Inventory,
	pub conditions: Conditions,
	pub hit_points: HitPoints,
	pub inspiration: bool,
	pub settings: Settings,
}
impl MutatorGroup for Persistent {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		for group in &self.named_groups.race {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.race_variant {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.lineage {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.upbringing {
			group.set_data_path(parent);
		}
		for group in &self.named_groups.background {
			group.set_data_path(parent);
		}
		for group in &self.classes {
			group.set_data_path(parent);
		}
		for group in &self.feats {
			group.set_data_path(parent);
		}
		self.inventory.set_data_path(parent);
		self.conditions.set_data_path(parent);
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		for (ability, score) in &self.ability_scores {
			stats
				.ability_scores_mut()
				.push_bonus(ability, (*score).into(), "Base Score".into());
		}
		stats.apply(&super::FinalizeAbilityScores.into(), parent);

		stats.apply(
			&AddMaxHitPoints {
				id: Some("Constitution x Levels".into()),
				value: Value::Evaluated(
					Product(vec![
						Value::Evaluated(GetLevel::default().into()),
						Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
					])
					.into(),
				),
			}
			.into(),
			parent,
		);

		for group in &self.named_groups.race {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.race_variant {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.lineage {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.upbringing {
			stats.apply_from(group, parent);
		}
		for group in &self.named_groups.background {
			stats.apply_from(group, parent);
		}
		for class in &self.classes {
			stats.apply_from(class, parent);
		}
		for feat in &self.feats {
			stats.add_feature(feat, parent);
		}
		stats.apply_from(&self.conditions, parent);
		stats.apply_from(&self.inventory, parent);
	}
}

impl Persistent {
	pub fn add_class(&mut self, mut class: Class) {
		class.levels.truncate(1);
		self.classes.push(class);
	}

	pub fn level(&self, class_name: Option<&str>) -> usize {
		match class_name {
			Some(class_name) => {
				for class in &self.classes {
					if class.name == class_name {
						return class.level_count();
					}
				}
				return 0;
			}
			None => self.classes.iter().map(|class| class.level_count()).sum(),
		}
	}

	pub fn hit_points(&self) -> &HitPoints {
		&self.hit_points
	}

	pub fn hit_points_mut(&mut self) -> &mut HitPoints {
		&mut self.hit_points
	}

	pub fn set_selected_value(&mut self, key: impl AsRef<Path>, value: impl Into<String>) {
		self.selected_values.set(key, value.into());
	}
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct HitPoints {
	pub current: u32,
	pub temp: u32,
	pub failure_saves: u8,
	pub success_saves: u8,
}
impl HitPoints {
	pub fn set_temp_hp(&mut self, value: u32) {
		self.temp = value;
	}

	pub fn plus_hp(mut self, amount: i32, max: u32) -> Self {
		let mut amt_abs = amount.abs() as u32;
		let had_hp = self.current > 0;
		match amount.signum() {
			1 => {
				self.current = self.current.saturating_add(amt_abs).min(max);
			}
			-1 if self.temp >= amt_abs => {
				self.temp = self.temp.saturating_sub(amt_abs);
			}
			-1 if self.temp < amt_abs => {
				amt_abs -= self.temp;
				self.temp = 0;
				self.current = self.current.saturating_sub(amt_abs);
			}
			_ => {}
		}
		if !had_hp && self.current != 0 {
			self.failure_saves = 0;
			self.success_saves = 0;
		}
		self
	}
}
impl std::ops::Add<(i32, u32)> for HitPoints {
	type Output = Self;

	fn add(self, (amount, max): (i32, u32)) -> Self::Output {
		self.plus_hp(amount, max)
	}
}
impl std::ops::AddAssign<(i32, u32)> for HitPoints {
	fn add_assign(&mut self, rhs: (i32, u32)) {
		*self = *self + rhs;
	}
}

#[derive(Clone)]
pub enum IdOrIndex {
	Id(Arc<SourceId>),
	Index(usize),
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Conditions {
	by_id: BTreeMap<SourceId, Condition>,
	custom: Vec<Condition>,
}
impl Conditions {
	pub fn insert(&mut self, condition: Condition) {
		match &condition.source_id {
			Some(id) => {
				self.by_id.insert(id.clone(), condition);
			}
			None => {
				self.custom.push(condition);
				self.custom.sort_by(|a, b| a.name.cmp(&b.name));
			}
		}
	}

	pub fn remove(&mut self, key: &IdOrIndex) {
		match key {
			IdOrIndex::Id(id) => {
				self.by_id.remove(&*id);
			}
			IdOrIndex::Index(idx) => {
				self.custom.remove(*idx);
			}
		}
	}

	pub fn iter(&self) -> impl Iterator<Item = &Condition> {
		self.by_id.values().chain(self.custom.iter())
	}

	pub fn iter_keyed(&self) -> impl Iterator<Item = (IdOrIndex, &Condition)> {
		let ids = self
			.by_id
			.iter()
			.map(|(id, value)| (IdOrIndex::Id(Arc::new(id.clone())), value));
		let indices = self
			.custom
			.iter()
			.enumerate()
			.map(|(idx, value)| (IdOrIndex::Index(idx), value));
		ids.chain(indices)
	}

	pub fn contains_id(&self, id: &SourceId) -> bool {
		self.by_id.contains_key(id)
	}
}
impl MutatorGroup for Conditions {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
		for condition in self.iter() {
			condition.set_data_path(parent);
		}
	}

	fn apply_mutators(&self, target: &mut Self::Target, parent: &Path) {
		for condition in self.iter() {
			target.apply_from(condition, parent);
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Settings {
	pub currency_auto_exchange: bool,
}
