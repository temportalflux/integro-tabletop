use super::PersonalityKind;
use crate::system::{
	dnd5e::{
		data::{
			action::AttackQuery,
			roll::{Modifier, Roll, RollSet},
			Ability, ArmorClass, DamageType, OtherProficiencies, Rest, Spell,
		},
		mutator::{Defense, Flag},
	},
	mutator::ReferencePath,
};
use enum_map::{enum_map, EnumMap};
use std::{
	collections::{BTreeMap, HashSet},
	path::{Path, PathBuf},
};

mod ability_score;
pub use ability_score::*;
mod actions;
pub use actions::*;
mod initiative;
pub use initiative::*;
mod object_cache;
pub use object_cache::*;
mod resource_depot;
pub use resource_depot::*;
mod saving_throw;
pub use saving_throw::*;
mod size;
pub use size::*;
mod skill;
pub use skill::*;
pub mod spellcasting;
pub use spellcasting::Spellcasting;
mod starting_equipment;
pub use starting_equipment::*;
mod stat;
pub use stat::*;

/// Data derived from the `Persistent`, such as bonuses to abilities/skills,
/// proficiencies, and actions. This data all lives within `Persistent` in
/// its various features and subtraits, and is compiled into one flat
/// structure for easy reference when displaying the character information.
#[derive(Clone, PartialEq, Debug)]
pub struct Derived {
	pub missing_selections: Vec<PathBuf>,
	pub ability_scores: AbilityScores,
	pub saving_throws: SavingThrows,
	pub skills: Skills,
	pub other_proficiencies: OtherProficiencies,
	pub speeds: Stat,
	pub senses: Stat,
	pub defenses: Defenses,
	pub max_hit_points: MaxHitPoints,
	pub initiative: Initiative,
	pub attack_bonuses: AttackBonuses,
	pub armor_class: ArmorClass,
	pub features: Features,
	pub description: DerivedDescription,
	pub flags: EnumMap<Flag, bool>,
	pub spellcasting: Spellcasting,
	pub starting_equipment: Vec<(Vec<StartingEquipment>, PathBuf)>,
	pub additional_objects: AdditionalObjectCache,
	pub rest_resets: RestResets,
	pub resource_depot: ResourceDepot,
}

impl Default for Derived {
	fn default() -> Self {
		Self {
			missing_selections: Default::default(),
			ability_scores: Default::default(),
			saving_throws: Default::default(),
			skills: Default::default(),
			other_proficiencies: Default::default(),
			speeds: Default::default(),
			senses: Default::default(),
			defenses: Default::default(),
			max_hit_points: Default::default(),
			initiative: Default::default(),
			attack_bonuses: Default::default(),
			armor_class: Default::default(),
			features: Default::default(),
			description: Default::default(),
			flags: enum_map! {
				Flag::ArmorStrengthRequirement => true,
			},
			spellcasting: Default::default(),
			starting_equipment: Default::default(),
			additional_objects: Default::default(),
			rest_resets: Default::default(),
			resource_depot: Default::default(),
		}
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Defenses(EnumMap<Defense, Vec<DefenseEntry>>);
#[derive(Clone, PartialEq, Debug)]
pub struct DefenseEntry {
	pub damage_type: Option<DamageType>,
	pub context: Option<String>,
	pub source: PathBuf,
}
impl Defenses {
	pub fn push(
		&mut self,
		kind: Defense,
		damage_type: Option<DamageType>,
		context: Option<String>,
		source: &ReferencePath,
	) {
		self.0[kind].push(DefenseEntry {
			damage_type,
			context,
			source: source.display.clone(),
		});
	}
}
impl std::ops::Deref for Defenses {
	type Target = EnumMap<Defense, Vec<DefenseEntry>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct DerivedDescription {
	pub life_expectancy: i32,
	pub size_formula: SizeFormula,
	pub personality_suggestions: EnumMap<PersonalityKind, Vec<String>>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct MaxHitPoints(i32, BTreeMap<PathBuf, i32>);
impl MaxHitPoints {
	pub fn push(&mut self, bonus: i32, source: &ReferencePath) {
		self.0 = self.0.saturating_add(bonus);
		self.1.insert(source.display.clone(), bonus);
	}

	pub fn value(&self) -> u32 {
		self.0.max(0) as u32
	}

	pub fn sources(&self) -> &BTreeMap<PathBuf, i32> {
		&self.1
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AttackBonuses {
	attack_roll: Vec<AttackRollBonus>,
	attack_damage: Vec<AttackDamageBonus>,
	attack_ability: Vec<AttackAbility>,
	spell_damage: Vec<SpellDamageBonus>,
}
#[derive(Clone, PartialEq, Debug)]
struct AttackRollBonus {
	bonus: i32,
	modifier: Option<Modifier>,
	queries: Vec<AttackQuery>,
	source: PathBuf,
}
#[derive(Clone, PartialEq, Debug)]
struct AttackDamageBonus {
	amount: Roll,
	damage_type: Option<DamageType>,
	queries: Vec<AttackQuery>,
	source: PathBuf,
}
#[derive(Clone, PartialEq, Debug)]
struct SpellDamageBonus {
	amount: Roll,
	queries: Vec<spellcasting::Filter>,
	source: PathBuf,
}
#[derive(Clone, PartialEq, Debug)]
struct AttackAbility {
	ability: Ability,
	queries: Vec<AttackQuery>,
	source: PathBuf,
}
impl AttackBonuses {
	pub fn add_to_weapon_attacks(&mut self, bonus: i32, queries: Vec<AttackQuery>, source: &ReferencePath) {
		self.attack_roll.push(AttackRollBonus {
			bonus,
			modifier: None,
			queries,
			source: source.display.clone(),
		});
	}

	pub fn modify_weapon_attacks(&mut self, modifier: Modifier, queries: Vec<AttackQuery>, source: &ReferencePath) {
		self.attack_roll.push(AttackRollBonus {
			bonus: 0,
			modifier: Some(modifier),
			queries,
			source: source.display.clone(),
		});
	}

	pub fn add_to_weapon_damage(
		&mut self,
		amount: Roll,
		damage_type: Option<DamageType>,
		queries: Vec<AttackQuery>,
		source: &ReferencePath,
	) {
		self.attack_damage.push(AttackDamageBonus {
			amount,
			damage_type,
			queries,
			source: source.display.clone(),
		});
	}

	pub fn add_ability_modifier(&mut self, ability: Ability, queries: Vec<AttackQuery>, source: &ReferencePath) {
		self.attack_ability.push(AttackAbility {
			ability,
			queries,
			source: source.display.clone(),
		});
	}

	pub fn add_to_spell_damage(&mut self, amount: Roll, queries: Vec<spellcasting::Filter>, source: &ReferencePath) {
		self.spell_damage.push(SpellDamageBonus {
			amount,
			queries,
			source: source.display.clone(),
		});
	}

	// TODO: This isn't used yet, and should be driving the attacks section of the ui
	pub fn get_weapon_attack(
		&self,
		action: &crate::system::dnd5e::data::action::Action,
	) -> Vec<(i32, Option<Modifier>, &Path)> {
		let mut bonuses = Vec::new();
		let Some(attack) = &action.attack else {
			return bonuses;
		};
		for bonus in &self.attack_roll {
			// Filter out any bonuses which do not meet the restriction
			'iter_query: for query in &bonus.queries {
				if query.is_attack_valid(attack) {
					bonuses.push((bonus.bonus, bonus.modifier, bonus.source.as_path()));
					break 'iter_query;
				}
			}
		}
		bonuses
	}

	pub fn get_weapon_damage(
		&self,
		action: &crate::system::dnd5e::data::action::Action,
	) -> Vec<(&Roll, &Option<DamageType>, &Path)> {
		let mut bonuses = Vec::new();
		let Some(attack) = &action.attack else {
			return bonuses;
		};
		for bonus in &self.attack_damage {
			// Filter out any bonuses which do not meet the restriction
			'iter_query: for query in &bonus.queries {
				if query.is_attack_valid(attack) {
					bonuses.push((&bonus.amount, &bonus.damage_type, bonus.source.as_path()));
					break 'iter_query;
				}
			}
		}
		bonuses
	}

	pub fn get_attack_ability_variants(&self, attack: &crate::system::dnd5e::data::action::Attack) -> HashSet<Ability> {
		// TODO: this doesnt report out the sources for the ability variants
		let mut abilities = HashSet::default();
		for bonus in &self.attack_ability {
			'iter_query: for query in &bonus.queries {
				if query.is_attack_valid(attack) {
					abilities.insert(bonus.ability);
					break 'iter_query;
				}
			}
		}
		abilities
	}

	pub fn get_spell_damage(&self, spell: &Spell) -> Vec<(&Roll, &Path)> {
		let mut bonuses = Vec::new();
		for bonus in &self.spell_damage {
			// Filter out any bonuses which do not meet the restriction
			'iter_query: for query in &bonus.queries {
				if query.matches(spell) {
					bonuses.push((&bonus.amount, bonus.source.as_path()));
					break 'iter_query;
				}
			}
		}
		bonuses
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct RestResets {
	entries: EnumMap<Rest, Vec<RestEntry>>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct RestEntry {
	pub restore_amount: Option<RollSet>,
	pub data_paths: Vec<PathBuf>,
	pub source: PathBuf,
}
impl RestResets {
	pub fn add(&mut self, rest: Rest, entry: RestEntry) {
		self.entries[rest].push(entry);
	}

	pub fn get(&self, rest: Rest) -> &Vec<RestEntry> {
		&self.entries[rest]
	}
}
