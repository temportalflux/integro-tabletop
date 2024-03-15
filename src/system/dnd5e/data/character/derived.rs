use super::{AttributedValue, PersonalityKind};
use crate::system::dnd5e::{
	data::{
		action::AttackQuery,
		proficiency,
		roll::{Modifier, Roll, RollSet},
		Ability, ArmorClass, DamageType, OtherProficiencies, Rest, Skill, Spell,
	},
	mutator::{Defense, Flag},
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
mod object_cache;
pub use object_cache::*;
mod sense;
pub use sense::*;
mod size;
pub use size::*;
mod speed;
pub use speed::*;
pub mod spellcasting;
pub use spellcasting::Spellcasting;
mod starting_equipment;
pub use starting_equipment::*;
mod resource_depot;
pub use resource_depot::*;

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
	pub speeds: Speeds,
	pub senses: Senses,
	pub defenses: Defenses,
	pub max_hit_points: MaxHitPoints,
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
pub struct SavingThrows {
	by_ability: EnumMap<Ability, ProficiencyModifiers>,
	general_modifiers: ModifierMap,
}
impl SavingThrows {
	pub fn add_proficiency(&mut self, ability: Ability, source: PathBuf) {
		self.by_ability[ability]
			.proficiency
			.push(proficiency::Level::Full, source);
	}

	pub fn add_modifier(
		&mut self,
		ability: Option<Ability>,
		modifier: Modifier,
		target: Option<String>,
		source: PathBuf,
	) {
		match ability {
			Some(ability) => &mut self.by_ability[ability].modifiers,
			None => &mut self.general_modifiers,
		}
		.insert(modifier, (target, source).into());
	}

	pub fn get_prof(&self, ability: Ability) -> &AttributedValue<proficiency::Level> {
		&self.by_ability[ability].proficiency
	}

	pub fn general_modifiers(&self) -> &ModifierMap {
		&self.general_modifiers
	}

	pub fn ability_modifiers(&self, ability: Ability) -> &ModifierMap {
		&self.by_ability[ability].modifiers
	}

	pub fn iter_modifiers(&self) -> impl Iterator<Item = (Option<Ability>, Modifier, &ModifierMapItem)> {
		self.by_ability
			.iter()
			.map(|(ability, saving_throw)| {
				saving_throw
					.modifiers
					.iter_all()
					.map(move |(modifier, item)| (Some(ability), modifier, item))
			})
			.flatten()
			.chain(
				self.general_modifiers
					.iter_all()
					.map(|(modifier, item)| (None, modifier, item)),
			)
	}
}
impl std::ops::Index<Ability> for SavingThrows {
	type Output = ProficiencyModifiers;
	fn index(&self, index: Ability) -> &Self::Output {
		&self.by_ability[index]
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ProficiencyModifiers {
	proficiency: AttributedValue<proficiency::Level>,
	modifiers: ModifierMap,
}
impl ProficiencyModifiers {
	pub fn proficiency(&self) -> &AttributedValue<proficiency::Level> {
		&self.proficiency
	}

	pub fn modifiers(&self) -> &ModifierMap {
		&self.modifiers
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ModifierMap {
	modifiers: EnumMap<Modifier, Vec<ModifierMapItem>>,
}
impl ModifierMap {
	pub fn insert(&mut self, modifier: Modifier, item: ModifierMapItem) {
		self.modifiers[modifier].push(item);
	}

	pub fn iter(&self) -> impl Iterator<Item = (Modifier, &Vec<ModifierMapItem>)> {
		self.modifiers.iter()
	}

	pub fn get(&self, modifier: Modifier) -> &Vec<ModifierMapItem> {
		&self.modifiers[modifier]
	}

	pub fn iter_all(&self) -> impl Iterator<Item = (Modifier, &ModifierMapItem)> {
		self.modifiers
			.iter()
			.map(|(modifier, items)| items.iter().map(move |item| (modifier, item)))
			.flatten()
	}
}
#[derive(Clone, Default, PartialEq, Debug)]
pub struct ModifierMapItem {
	pub context: Option<String>,
	pub source: PathBuf,
}
impl From<(Option<String>, PathBuf)> for ModifierMapItem {
	fn from((context, source): (Option<String>, PathBuf)) -> Self {
		Self { context, source }
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Skills {
	ability_modifiers: EnumMap<Ability, ModifierMap>,
	skills: EnumMap<Skill, ProficiencyModifiers>,
}
impl Skills {
	pub fn add_proficiency(&mut self, skill: Skill, level: proficiency::Level, source: PathBuf) {
		self.skills[skill].proficiency.push(level, source);
	}

	pub fn add_ability_modifier(
		&mut self,
		ability: Ability,
		modifier: Modifier,
		context: Option<String>,
		source: PathBuf,
	) {
		self.ability_modifiers[ability].insert(modifier, (context, source).into());
	}

	pub fn add_skill_modifier(&mut self, skill: Skill, modifier: Modifier, context: Option<String>, source: PathBuf) {
		self.skills[skill].modifiers.insert(modifier, (context, source).into());
	}

	pub fn proficiency(&self, skill: Skill) -> &AttributedValue<proficiency::Level> {
		self.skills[skill].proficiency()
	}

	pub fn ability_modifiers(&self, ability: Ability) -> &ModifierMap {
		&self.ability_modifiers[ability]
	}

	pub fn skill_modifiers(&self, skill: Skill) -> &ModifierMap {
		&self.skills[skill].modifiers
	}

	pub fn iter_ability_modifiers(&self, ability: Ability) -> impl Iterator<Item = (Modifier, &Vec<ModifierMapItem>)> {
		self.ability_modifiers[ability].iter()
	}

	pub fn iter_skill_modifiers(&self, skill: Skill) -> impl Iterator<Item = (Modifier, &Vec<ModifierMapItem>)> {
		self.ability_modifiers[skill.ability()]
			.iter()
			.chain(self.skills[skill].modifiers().iter())
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
	pub fn push(&mut self, kind: Defense, damage_type: Option<DamageType>, context: Option<String>, source: PathBuf) {
		self.0[kind].push(DefenseEntry {
			damage_type,
			context,
			source,
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
	pub fn push(&mut self, bonus: i32, source: PathBuf) {
		self.0 = self.0.saturating_add(bonus);
		self.1.insert(source, bonus);
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
	pub fn add_to_weapon_attacks(&mut self, bonus: i32, queries: Vec<AttackQuery>, source: PathBuf) {
		self.attack_roll.push(AttackRollBonus { bonus, queries, source });
	}

	pub fn add_to_weapon_damage(
		&mut self,
		amount: Roll,
		damage_type: Option<DamageType>,
		queries: Vec<AttackQuery>,
		source: PathBuf,
	) {
		self.attack_damage.push(AttackDamageBonus {
			amount,
			damage_type,
			queries,
			source,
		});
	}

	pub fn add_ability_modifier(&mut self, ability: Ability, queries: Vec<AttackQuery>, source: PathBuf) {
		self.attack_ability.push(AttackAbility {
			ability,
			queries,
			source,
		});
	}

	pub fn add_to_spell_damage(&mut self, amount: Roll, queries: Vec<spellcasting::Filter>, source: PathBuf) {
		self.spell_damage.push(SpellDamageBonus {
			amount,
			queries,
			source,
		});
	}

	pub fn get_weapon_attack(&self, action: &crate::system::dnd5e::data::action::Action) -> Vec<(i32, &Path)> {
		let mut bonuses = Vec::new();
		let Some(attack) = &action.attack else {
			return bonuses;
		};
		for bonus in &self.attack_roll {
			// Filter out any bonuses which do not meet the restriction
			'iter_query: for query in &bonus.queries {
				if query.is_attack_valid(attack) {
					bonuses.push((bonus.bonus, bonus.source.as_path()));
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
