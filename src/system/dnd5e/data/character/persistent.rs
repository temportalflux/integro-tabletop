use crate::{
	path_map::PathMap,
	system::dnd5e::data::{
		character::Character, condition::BoxedCondition, item, Ability, Background, BoxedFeature,
		Class, Description, Lineage, Score, Upbringing,
	},
	utility::MutatorGroup,
};
use enum_map::EnumMap;

/// Core character data which is (de)serializable and
/// from which the derived data can be compiled.
#[derive(Clone, PartialEq, Default)]
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
	pub inspiration: bool,
}
impl MutatorGroup for Persistent {
	type Target = Character;

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

	pub fn temp_hp_mut(&mut self) -> &mut u32 {
		&mut self.hit_points.1
	}

	pub fn add_hp(&mut self, amount: u32, max: u32) {
		self.hit_points.0 = self.hit_points.0.saturating_add(amount).min(max);
	}

	pub fn sub_hp(&mut self, mut amount: u32) {
		if self.hit_points.1 >= amount {
			self.hit_points.1 = self.hit_points.1.saturating_sub(amount);
			return;
		}
		else {
			amount -= self.hit_points.1;
			self.hit_points.1 = 0;
		}
		self.hit_points.0 = self.hit_points.0.saturating_sub(amount);
	}
}
