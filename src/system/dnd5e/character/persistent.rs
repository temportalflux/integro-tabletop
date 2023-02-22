use crate::{
	path_map::PathMap,
	system::dnd5e::{
		character::*, condition::BoxedCondition, item, mutator, Ability, BoxedFeature, Score,
	},
};
use enum_map::EnumMap;

/// Core character data which is (de)serializable and
/// from which the derived data can be compiled.
#[derive(Clone, PartialEq)]
pub struct Persistent {
	pub lineages: [Option<Lineage>; 2],
	pub upbringing: Option<Upbringing>,
	pub background: Option<Background>,
	pub classes: Vec<Class>,
	pub feats: Vec<BoxedFeature>,
	pub description: Description,
	pub ability_scores: EnumMap<Ability, Score>,
	pub selected_values: PathMap<String>,
	pub inventory: item::Inventory,
	pub conditions: Vec<BoxedCondition>,
	pub hit_points: (u32, u32),
}
impl mutator::Container for Persistent {
	fn apply_mutators<'c>(&self, stats: &mut Character) {
		for lineage in &self.lineages {
			if let Some(lineage) = lineage {
				stats.apply_from(lineage);
			}
		}
		if let Some(upbringing) = &self.upbringing {
			stats.apply_from(upbringing);
		}
		if let Some(background) = &self.background {
			stats.apply_from(background);
		}
		for class in &self.classes {
			stats.apply_from(class);
		}
		for feat in &self.feats {
			stats.add_feature(feat);
		}
		stats.apply_from(&self.inventory);
	}
}

impl Persistent {
	pub fn level(&self, class_name: Option<&str>) -> usize {
		match class_name {
			Some(class_name) => {
				let Ok(class_idx) = self.classes.binary_search_by(|class| class.name.as_str().cmp(class_name)) else { return 0; };
				self.classes.get(class_idx).unwrap().level_count()
			}
			None => self.classes.iter().map(|class| class.level_count()).sum(),
		}
	}
}
