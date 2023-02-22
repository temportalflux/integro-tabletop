use super::{
	AbilityScores, ArmorClass, Defenses, Derived, DerivedDescription, OtherProficiencies,
	Persistent, SavingThrows, Senses, Skills, Speeds,
};
use crate::{
	path_map::PathMap,
	system::dnd5e::{
		action::Action, criteria::BoxedCriteria, item, mutator, proficiency, Ability, BoxedFeature,
		Score,
	},
};
use enum_map::Enum;
use enumset::EnumSetType;
use std::{
	path::{Path, PathBuf},
	rc::Rc,
	str::FromStr,
};

/// The pairing of `Character` and `Derived` to form a singlular reference
/// structure for all character data.
#[derive(Clone, PartialEq)]
pub struct Character {
	character: Persistent,
	derived: Derived,
	source_path: SourcePath,
}
impl From<Persistent> for Character {
	fn from(character: Persistent) -> Self {
		let mut full = Self {
			character,
			derived: Derived::default(),
			source_path: SourcePath::default(),
		};
		full.generate_derived();
		full
	}
}
impl yew::Reducible for Character {
	type Action = yew::Callback<Self, Self>;

	fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
		Rc::new(action.emit((*self).clone()))
	}
}
impl Character {
	pub fn generate_derived(&mut self) {
		self.derived = Derived::default();
		self.apply_from(&self.character.clone());
	}

	pub fn apply_from(&mut self, container: &impl mutator::Container) {
		let scope = self
			.source_path
			.push(container.id(), container.display_id());
		container.apply_mutators(self);
		self.source_path.pop(scope);
	}

	pub fn apply(&mut self, mutator: &mutator::BoxedMutator) {
		let scope = self.source_path.push(mutator.id(), true);
		mutator.apply(self);
		self.source_path.pop(scope);
	}

	pub fn source_path(&self) -> PathBuf {
		self.source_path.to_display()
	}

	fn get_selections_at(&self, path: &SourcePath) -> Option<&Vec<String>> {
		self.character.selected_values.get(&path.to_data())
	}

	pub fn resolve_selector<T>(&mut self, selector: &mutator::Selector<T>) -> Option<T>
	where
		T: Clone + 'static + FromStr,
	{
		if let mutator::Selector::Specific(value) = selector {
			return Some(value.clone());
		}
		let scope = self.source_path.push(selector.id(), false);
		let selections = self.get_selections_at(&self.source_path);
		let value = match selections {
			Some(selections) => match selections.first() {
				Some(selected) => T::from_str(&selected).ok(),
				None => None,
			},
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
		criteria.evaluate(&self.character)
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

	/// Returns the score/value for a given ability. Any bonuses beyond the character's base scores
	/// are provided with a path to the feature which provided that bonus.
	pub fn ability_score(&self, ability: Ability) -> (Score, Vec<(PathBuf, i32)>) {
		let mut score = self.character.ability_scores[ability];
		let original_score = score.0;
		let attributed = self.derived.ability_scores.get(ability);
		(*score) += attributed.value;
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
		match self.character.level(None) {
			1..=4 => 2,
			5..=8 => 3,
			9..=12 => 4,
			13..=16 => 5,
			17.. => 6,
			_ => 0,
		}
	}

	pub fn initiative_bonus(&self) -> i32 {
		self.ability_modifier(Ability::Dexterity, None)
	}

	pub fn hit_points(&self, kind: HitPoint) -> u32 {
		match kind {
			HitPoint::Current => self.character.hit_points.0,
			HitPoint::Max => self.derived.max_hit_points,
			HitPoint::Temp => self.character.hit_points.1,
		}
	}

	pub fn hit_points_mut(&mut self, kind: HitPoint) -> &mut u32 {
		match kind {
			HitPoint::Current => &mut self.character.hit_points.0,
			HitPoint::Max => &mut self.derived.max_hit_points,
			HitPoint::Temp => &mut self.character.hit_points.1,
		}
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

#[derive(Clone, PartialEq, Default)]
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
