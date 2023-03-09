use crate::{
	path_map::PathMap,
	system::dnd5e::{
		data::{
			action::{Action, ActionSource, AttackCheckKind},
			character::{
				AbilityScores, Defenses, Derived, DerivedDescription, MaxHitPoints, Persistent,
				SavingThrows, Senses, Skills, Speeds,
			},
			item::{self, weapon, ItemKind},
			mutator::Flag,
			proficiency, Ability, ArmorClass, BoxedFeature, OtherProficiencies, Score,
		},
		BoxedCriteria, BoxedMutator,
	},
	utility::{Dependencies, MutatorGroup, Selector},
};
use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use std::{
	path::{Path, PathBuf},
	rc::Rc,
	str::FromStr,
};

use super::HitPoints;

#[derive(Clone, Copy, PartialEq)]
pub enum ActionEffect {
	Recompile,
}

/// The pairing of `Character` and `Derived` to form a singlular reference
/// structure for all character data.
#[derive(Clone, PartialEq, Debug)]
pub struct Character {
	character: Persistent,
	derived: Derived,
	mutators: Vec<MutatorEntry>,
	source_path: SourcePath,
}
#[derive(Clone, PartialEq, Debug)]
struct MutatorEntry {
	node_id: &'static str,
	dependencies: Dependencies,
	mutator: BoxedMutator,
	source: SourcePath,
}
impl From<Persistent> for Character {
	fn from(character: Persistent) -> Self {
		let mut full = Self {
			character,
			derived: Derived::default(),
			mutators: Vec::new(),
			source_path: SourcePath::default(),
		};
		full.apply_from(&full.character.clone());
		full.apply_cached_mutators();
		full
	}
}
impl yew::Reducible for Character {
	type Action = Box<dyn FnOnce(&mut Persistent, &Rc<Self>) -> Option<ActionEffect>>;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		let mut full = (*self).clone();
		Rc::new(match action(&mut full.character, &self) {
			None => full,
			Some(ActionEffect::Recompile) => {
				let mut updated = Self::new(full.character.clone());
				updated.apply_from(&full.character);
				updated.apply_cached_mutators();
				updated
			}
		})
	}
}
impl Character {
	pub fn new(character: Persistent) -> Self {
		Self {
			character,
			derived: Derived::default(),
			mutators: Vec::new(),
			source_path: SourcePath::default(),
		}
	}

	pub fn apply_from(&mut self, container: &impl MutatorGroup<Target = Self>) {
		let scope = self
			.source_path
			.push(container.id(), container.display_id());
		container.apply_mutators(self);
		self.source_path.pop(scope);
	}

	pub fn apply(&mut self, mutator: &BoxedMutator) {
		let scope = self.source_path.push(mutator.data_id(), true);
		self.insert_mutator(MutatorEntry {
			node_id: mutator.get_node_name(),
			dependencies: mutator.dependencies(),
			mutator: mutator.clone(),
			source: self.source_path.clone(),
		});
		self.source_path.pop(scope);
	}

	fn insert_mutator(&mut self, entry: MutatorEntry) {
		let idx = self.mutators.binary_search_by(|value| {
			match (&*value.dependencies, &*entry.dependencies) {
				// neither have dependencies, so they are considered equal, might as well order by node_id.
				(None, None) => value.node_id.cmp(&entry.node_id),
				// existing element has dependencies, but the new one doesn't. New one goes before existing.
				(Some(_), None) => std::cmp::Ordering::Greater,
				// new entry has dependencies, all deps in new must come before it
				(_, Some(deps)) => match deps.contains(&value.node_id) {
					// existing is not required for new, might as well sort by node_id.
					false => value.node_id.cmp(&entry.node_id),
					// existing is required, it must come before the new entry
					true => std::cmp::Ordering::Greater,
				},
			}
		});
		let idx = match idx {
			Ok(idx) => idx,
			Err(idx) => idx,
		};
		self.mutators.insert(idx, entry);
	}

	fn apply_cached_mutators(&mut self) {
		let mutators = self.mutators.drain(..).collect::<Vec<_>>();
		for entry in mutators.into_iter() {
			log::debug!(
				target: "character",
				"applying mutator: {:?} deps:{:?}",
				entry.node_id,
				entry.dependencies
			);
			self.source_path = entry.source;
			entry.mutator.apply(self);
		}
	}

	pub fn source_path(&self) -> PathBuf {
		self.source_path.to_display()
	}

	fn get_selections_at(&self, path: impl AsRef<Path>) -> Option<&Vec<String>> {
		self.character.selected_values.get(path.as_ref())
	}

	pub fn get_first_selection_at<T>(
		&self,
		data_path: impl AsRef<Path>,
	) -> Option<Result<T, <T as FromStr>::Err>>
	where
		T: Clone + 'static + FromStr,
	{
		let selections = self.get_selections_at(data_path);
		selections
			.map(|all| all.first())
			.flatten()
			.map(|selected| T::from_str(&selected))
	}

	pub fn resolve_selector<T>(&mut self, selector: &Selector<T>) -> Option<T>
	where
		T: Clone + 'static + FromStr,
	{
		if let Selector::Specific(value) = selector {
			return Some(value.clone());
		}
		let scope = self.source_path.push(selector.id(), false);
		let value = match self.get_first_selection_at::<T>(&self.source_path.to_data()) {
			Some(Ok(value)) => Some(value),
			Some(Err(_)) => None,
			None => {
				self.derived
					.missing_selections
					.push(self.source_path.to_data());
				None
			}
		};
		self.source_path.pop(scope);
		value
	}
}

impl Character {
	pub fn evaluate(&self, criteria: &BoxedCriteria) -> Result<(), String> {
		criteria.evaluate(&self)
	}

	pub fn selected_values_in(&self, parent: impl AsRef<Path>) -> Option<&PathMap<String>> {
		self.character.selected_values.get_all(parent)
	}

	pub fn missing_selections_in(&self, parent: impl AsRef<Path>) -> Vec<&Path> {
		self.derived
			.missing_selections
			.iter()
			.filter_map(|path| path.strip_prefix(&parent).ok())
			.collect::<Vec<_>>()
	}

	pub fn persistent(&self) -> &Persistent {
		&self.character
	}

	pub fn flags(&self) -> &EnumMap<Flag, bool> {
		&self.derived.flags
	}

	pub fn flags_mut(&mut self) -> &mut EnumMap<Flag, bool> {
		&mut self.derived.flags
	}

	/// Returns the score/value for a given ability. Any bonuses beyond the character's base scores
	/// are provided with a path to the feature which provided that bonus.
	pub fn ability_score(&self, ability: Ability) -> (Score, Vec<(PathBuf, i32)>) {
		let mut score = self.character.ability_scores[ability];
		let original_score = *score as i32;
		let attributed = self.derived.ability_scores.get(ability);
		*score = original_score.saturating_add(attributed.value) as u32;
		let mut sources = attributed.sources.clone();
		sources.insert(0, ("".into(), original_score));
		(score, sources)
	}

	pub fn ability_modifier(
		&self,
		ability: Ability,
		proficiency: Option<proficiency::Level>,
	) -> i32 {
		let modifier = self.ability_score(ability).0.modifier();
		let bonus = match proficiency {
			Some(proficiency) => {
				let prof_bonus_multiplier = match proficiency {
					proficiency::Level::None => 0.0,
					proficiency::Level::Half => 0.5,
					proficiency::Level::Full => 1.0,
					proficiency::Level::Double => 2.0,
				};
				((self.proficiency_bonus() as f32) * prof_bonus_multiplier).floor() as i32
			}
			None => 0,
		};
		modifier + bonus
	}

	pub fn ability_scores_mut(&mut self) -> &mut AbilityScores {
		&mut self.derived.ability_scores
	}

	pub fn saving_throws(&self) -> &SavingThrows {
		&self.derived.saving_throws
	}

	pub fn saving_throws_mut(&mut self) -> &mut SavingThrows {
		&mut self.derived.saving_throws
	}

	pub fn skills(&self) -> &Skills {
		&self.derived.skills
	}

	pub fn skills_mut(&mut self) -> &mut Skills {
		&mut self.derived.skills
	}

	pub fn armor_class(&self) -> &ArmorClass {
		&self.derived.armor_class
	}

	pub fn armor_class_mut(&mut self) -> &mut ArmorClass {
		&mut self.derived.armor_class
	}

	pub fn speeds(&self) -> &Speeds {
		&self.derived.speeds
	}

	pub fn speeds_mut(&mut self) -> &mut Speeds {
		&mut self.derived.speeds
	}

	pub fn senses(&self) -> &Senses {
		&self.derived.senses
	}

	pub fn senses_mut(&mut self) -> &mut Senses {
		&mut self.derived.senses
	}

	pub fn level(&self, class_name: Option<&str>) -> usize {
		self.character.level(class_name)
	}

	pub fn proficiency_bonus(&self) -> i32 {
		proficiency::proficiency_bonus(self.character.level(None))
	}

	pub fn initiative_bonus(&self) -> i32 {
		self.ability_modifier(Ability::Dexterity, None)
	}

	pub fn inspiration(&self) -> bool {
		self.character.inspiration
	}

	pub fn get_hp(&self, kind: HitPoint) -> u32 {
		match kind {
			HitPoint::Current => self.character.hit_points.current,
			HitPoint::Max => self.derived.max_hit_points.value(),
			HitPoint::Temp => self.character.hit_points.temp,
		}
	}

	pub fn max_hit_points(&self) -> &MaxHitPoints {
		&self.derived.max_hit_points
	}

	pub fn max_hit_points_mut(&mut self) -> &mut MaxHitPoints {
		&mut self.derived.max_hit_points
	}

	pub fn hit_points(&self) -> &HitPoints {
		self.character.hit_points()
	}

	pub fn hit_points_mut(&mut self) -> &mut HitPoints {
		self.character.hit_points_mut()
	}

	pub fn defenses(&self) -> &Defenses {
		&self.derived.defenses
	}

	pub fn defenses_mut(&mut self) -> &mut Defenses {
		&mut self.derived.defenses
	}

	pub fn other_proficiencies(&self) -> &OtherProficiencies {
		&self.derived.other_proficiencies
	}

	pub fn other_proficiencies_mut(&mut self) -> &mut OtherProficiencies {
		&mut self.derived.other_proficiencies
	}

	pub fn add_feature(&mut self, feature: &BoxedFeature) {
		self.derived
			.features
			.insert(&self.source_path.to_display(), feature.clone());
		self.apply_from(feature.inner());
	}

	pub fn features(&self) -> &PathMap<BoxedFeature> {
		&self.derived.features
	}

	pub fn actions(&self) -> &Vec<Action> {
		&self.derived.actions
	}

	pub fn actions_mut(&mut self) -> &mut Vec<Action> {
		&mut self.derived.actions
	}

	pub fn inventory(&self) -> &item::Inventory {
		&self.character.inventory
	}

	pub fn inventory_mut(&mut self) -> &mut item::Inventory {
		&mut self.character.inventory
	}

	pub fn iter_actions_mut_for(
		&mut self,
		restriction: &Option<weapon::Restriction>,
	) -> Vec<&mut Action> {
		let mut actions = self.derived.actions.iter_mut().collect::<Vec<_>>();
		if let Some(weapon::Restriction {
			weapon_kind,
			attack_kind,
			ability,
		}) = restriction
		{
			if !weapon_kind.is_empty() {
				actions = actions
					.into_iter()
					.filter_map(|action| {
						let Some(ActionSource::Item(item_id)) = &action.source else { return None; };
						let Some(item) = self.character.inventory.get_item(item_id) else { return None; };
						let ItemKind::Equipment(equipment) = &item.kind else { return None; };
						let Some(weapon) = &equipment.weapon else { return None; };
						weapon_kind.contains(&weapon.kind).then_some(action)
					})
					.collect();
			}

			if !attack_kind.is_empty() {
				actions = actions
					.into_iter()
					.filter_map(|action| {
						let Some(attack) = &action.attack else { return None; };
						attack_kind.contains(&attack.kind.kind()).then_some(action)
					})
					.collect();
			}

			if !ability.is_empty() {
				actions = actions.into_iter().filter_map(|action| {
					let Some(attack) = &action.attack else { return None; };
					let AttackCheckKind::AttackRoll { ability: atk_roll_ability, .. } = &attack.check else { return None; };
					ability.contains(atk_roll_ability).then_some(action)
				}).collect();
			}
		}
		actions
	}

	pub fn derived_description_mut(&mut self) -> &mut DerivedDescription {
		&mut self.derived.description
	}
}

#[derive(Debug, EnumSetType, Enum)]
pub enum HitPoint {
	Current,
	Max,
	Temp,
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct SourcePath {
	display: PathBuf,
	data: PathBuf,
}

impl SourcePath {
	fn push<P: AsRef<Path>>(&mut self, path: Option<P>, include_display: bool) -> Scope {
		let Some(path) = path else { return Scope::NoChange; };
		self.data.push(&path);
		if !include_display {
			return Scope::DataOnly;
		}
		self.display.push(&path);
		Scope::All
	}

	fn pop(&mut self, scope: Scope) {
		if scope == Scope::NoChange {
			return;
		}
		self.data.pop();
		if scope == Scope::DataOnly {
			return;
		}
		self.display.pop();
	}

	fn adjusted_path(&self, path: &PathBuf) -> PathBuf {
		match std::path::MAIN_SEPARATOR {
			'/' => path.clone(),
			_ => PathBuf::from(
				path.iter()
					.map(|s| s.to_str().unwrap())
					.collect::<Vec<_>>()
					.join("/"),
			),
		}
	}

	pub fn to_display(&self) -> PathBuf {
		self.adjusted_path(&self.display)
	}

	fn to_data(&self) -> PathBuf {
		self.adjusted_path(&self.data)
	}
}

#[derive(PartialEq, Eq)]
enum Scope {
	NoChange,
	DataOnly,
	All,
}
