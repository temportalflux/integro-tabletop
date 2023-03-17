use super::AttributedValue;
use crate::{
	path_map::PathMap,
	system::dnd5e::{
		data::{
			action::{Action, ActivationKind, Attack, AttackCheckKind, AttackKindValue},
			mutator::{Defense, Flag},
			proficiency,
			roll::{Modifier, RollSet},
			Ability, ArmorClass, BoxedFeature, DamageRoll, DamageType, OtherProficiencies, Skill,
		},
		Value,
	},
};
use enum_map::{enum_map, EnumMap};
use itertools::Itertools;
use std::{
	collections::{BTreeMap, HashSet},
	path::PathBuf,
};

mod sense;
pub use sense::*;
mod speed;
pub use speed::*;

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
	pub features: PathMap<BoxedFeature>,
	pub max_hit_points: MaxHitPoints,
	pub armor_class: ArmorClass,
	pub actions: Vec<Action>,
	pub description: DerivedDescription,
	pub flags: EnumMap<Flag, bool>,
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
			features: Default::default(),
			max_hit_points: Default::default(),
			armor_class: Default::default(),
			actions: vec![Action {
				name: "Unarmed Strike".into(),
				description: "Instead of using a weapon to make a melee weapon attack, \
				you can use an unarmed strike: a punch, kick, head-butt, or similar \
				forceful blow (none of which count as weapons). On a hit, an unarmed \
				strike deals bludgeoning damage equal to 1 + your Strength modifier. \
				You are proficient with your unarmed strikes."
					.into(),
				activation_kind: ActivationKind::Action,
				attack: Some(Attack {
					kind: AttackKindValue::Melee { reach: 5 },
					check: AttackCheckKind::AttackRoll {
						ability: Ability::Strength,
						proficient: Value::Fixed(true),
					},
					area_of_effect: None,
					damage: Some(DamageRoll {
						base_bonus: 1,
						damage_type: DamageType::Bludgeoning,
						..Default::default()
					}),
				}),
				..Default::default()
			}],
			description: Default::default(),
			flags: enum_map! {
				Flag::ArmorStrengthRequirement => true,
			},
		}
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AbilityScores(EnumMap<Ability, AttributedValue<i32>>);
impl AbilityScores {
	pub fn push_bonus(&mut self, ability: Ability, bonus: i32, source: PathBuf) {
		self.0[ability].push(bonus, source);
	}

	pub fn get(&self, ability: Ability) -> &AttributedValue<i32> {
		&self.0[ability]
	}
}

// Ability Scores
// - have a default max of 20
// - base value is in persistent data
// - mutator can extend the max to some value
// - mutator can add a bonus
// - mutator can add a bonus with a constraint (e.g. total score <= x)
struct AbilityScoreItem {
	bonuses: Vec<(Bonus, PathBuf)>,
	max_score: u32,
	max_score_incs: Vec<(u32, PathBuf)>,
}
impl AbilityScoreItem {
	fn from_score(score: u32) -> Self {
		Self {
			bonuses: vec![(
				Bonus {
					value: score,
					max_total: None,
				},
				"Base Score".into(),
			)],
			max_score: 20,
			max_score_incs: vec![],
		}
	}

	/// Returns the evaluated total scoree, and the list of paths that were used from the list of bonuses.
	fn evaluate(&self) -> (u32, Vec<&std::path::Path>) {
		let max_value = self.max_score + self.max_score_incs.iter().map(|(v, _)| v).sum::<u32>();
		let (no_constraints, constrained): (Vec<_>, Vec<_>) = self
			.bonuses
			.iter()
			.enumerate()
			.partition_map(|(idx, (bonus, _))| match bonus.max_total {
				None => itertools::Either::Left((idx, bonus.value)),
				Some(max_total) => itertools::Either::Right((idx, (bonus.value, max_total))),
			});
		let (mut used_indices, unconstrained): (HashSet<_>, Vec<_>) =
			no_constraints.into_iter().unzip();
		let total = unconstrained.into_iter().sum::<u32>();
		let (total, additional_indices) = optimize_max_sums(total, constrained);
		if let Some(indices) = additional_indices {
			used_indices.extend(indices.into_iter());
		}
		let used_paths = used_indices
			.into_iter()
			.filter_map(|idx| self.bonuses.get(idx))
			.map(|(_, path)| path.as_path())
			.collect::<Vec<_>>();
		(total.min(max_value), used_paths)
	}
}

struct Bonus {
	value: u32,
	max_total: Option<u32>,
}

fn optimize_max_sums(
	base: u32,
	mut bonuses: Vec<(/*idx*/ usize, (/*bonus*/ u32, /*max*/ u32))>,
) -> (u32, Option<Vec<usize>>) {
	use itertools::Itertools;

	// Lower Extents: drop all entries whose max constraint will never be met (req max < base)
	bonuses.retain(|(_, (_, max))| *max >= base);

	// The largest possible total which satisfies all constraints
	let mut max_valid_total = base;
	// The indices of the entries used to make the total
	let mut best_set_indices = None;
	// Optimization problem
	// Find the optimal use of bonuses which maximizes the total value
	// and where all included bonuses have a max >= the final total.
	// NOTE: brute force - this has very poor performance
	//println!("base={base} relevant-bonuses={bonuses:?}");
	for size in 0..bonuses.len() {
		for bonuses in bonuses.iter().combinations(size + 1) {
			let (indices, bonuses): (Vec<_>, Vec<_>) = bonuses.into_iter().cloned().unzip();
			//println!("\tCombination: {bonuses:?}");
			let (bonuses, constraints): (Vec<_>, Vec<_>) = bonuses.into_iter().unzip();
			let total = base + bonuses.into_iter().sum::<u32>();
			// filter out any constrains which are met (the max >= this combination's total)
			// any remaining are unmet constraints (max < total).
			// if there are no remaining, then this is a valid combination.
			let num_constraints_unmet = constraints.into_iter().filter(|max| *max < total).count();
			//println!("\t\ttotal={total} valid? {}", num_constraints_unmet == 0);
			if num_constraints_unmet == 0 {
				if total > max_valid_total {
					max_valid_total = total;
					best_set_indices = Some(indices);
				}
			}
		}
	}

	(max_valid_total, best_set_indices)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn oms_empty() {
		let value = optimize_max_sums(10, vec![]);
		assert_eq!(value, (10, None));
	}

	#[test]
	fn oms_only_lower() {
		let value = optimize_max_sums(15, vec![(0, (1, 11)), (1, (5, 14))]);
		assert_eq!(value, (15, None));
	}

	#[test]
	fn oms_no_optimization() {
		let value = optimize_max_sums(14, vec![(0, (2, 13)), (1, (4, 21))]);
		assert_eq!(value, (18, Some(vec![1])));
	}

	#[test]
	fn oms_no_extents() {
		let value = optimize_max_sums(
			9,
			vec![
				(0, (1, 14)),
				(1, (2, 12)), // should drop this
				(2, (1, 15)),
				(3, (5, 17)),
			],
		);
		assert_eq!(value, (15, Some(vec![2, 3])));
	}

	#[test]
	fn oms_mixed() {
		let value = optimize_max_sums(
			8,
			vec![
				// lower extent (max < base)
				(0, (2, 5)),
				// to optimize
				(1, (2, 15)),
				(2, (3, 15)),
				(3, (1, 16)),
				(4, (3, 17)),
				(5, (4, 18)),
			],
		);
		assert_eq!(value, (16, Some(vec![3, 4, 5])));
	}

	#[test]
	fn oms_same_max() {
		let value = optimize_max_sums(
			8,
			vec![
				(0, (1, 17)),
				(1, (2, 17)),
				(2, (3, 17)),
				(3, (4, 17)),
				(4, (5, 17)),
			],
		);
		assert_eq!(value, (17, Some(vec![3, 4])));
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct SavingThrows {
	by_ability: EnumMap<
		Ability,
		(
			/*is proficient*/ AttributedValue<proficiency::Level>,
			/*adv modifiers*/ Vec<(Option<String>, PathBuf)>,
		),
	>,
	general_modifiers: Vec<(Option<String>, PathBuf)>,
}
impl SavingThrows {
	pub fn add_proficiency(&mut self, ability: Ability, source: PathBuf) {
		self.by_ability[ability]
			.0
			.push(proficiency::Level::Full, source);
	}

	pub fn add_modifier(
		&mut self,
		ability: Option<Ability>,
		target: Option<String>,
		source: PathBuf,
	) {
		match ability {
			Some(ability) => &mut self.by_ability[ability].1,
			None => &mut self.general_modifiers,
		}
		.push((target, source));
	}

	pub fn get_prof(&self, ability: Ability) -> &AttributedValue<proficiency::Level> {
		&self.by_ability[ability].0
	}

	pub fn iter_modifiers(
		&self,
	) -> impl Iterator<Item = (Option<Ability>, &Option<String>, &PathBuf)> {
		self.by_ability
			.iter()
			.map(|(ability, (_, modifiers))| {
				modifiers
					.iter()
					.map(move |(target, path)| (Some(ability), target, path))
			})
			.flatten()
			.chain(
				self.general_modifiers
					.iter()
					.map(|(target, path)| (None, target, path)),
			)
	}
}
impl std::ops::Index<Ability> for SavingThrows {
	type Output = (
		AttributedValue<proficiency::Level>,
		Vec<(Option<String>, PathBuf)>,
	);
	fn index(&self, index: Ability) -> &Self::Output {
		&self.by_ability[index]
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Skills(
	EnumMap<
		Skill,
		(
			/*proficiency*/ AttributedValue<proficiency::Level>,
			/*modifiers*/
			EnumMap<Modifier, Vec<(/*context*/ Option<String>, /*source*/ PathBuf)>>,
		),
	>,
);
impl Skills {
	pub fn add_proficiency(&mut self, skill: Skill, level: proficiency::Level, source: PathBuf) {
		self.0[skill].0.push(level, source);
	}

	pub fn add_modifier(
		&mut self,
		skill: Skill,
		modifier: Modifier,
		context: Option<String>,
		source: PathBuf,
	) {
		self.0[skill].1[modifier].push((context, source));
	}
}
impl std::ops::Index<Skill> for Skills {
	type Output = (
		/*proficiency*/ AttributedValue<proficiency::Level>,
		/*modifiers*/
		EnumMap<Modifier, Vec<(/*context*/ Option<String>, /*source*/ PathBuf)>>,
	);

	fn index(&self, index: Skill) -> &Self::Output {
		&self.0[index]
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Defenses(EnumMap<Defense, Vec<DefenseEntry>>);
#[derive(Clone, PartialEq, Debug)]
pub struct DefenseEntry {
	pub damage_type: Option<Value<DamageType>>,
	pub context: Option<String>,
	pub source: PathBuf,
}
impl Defenses {
	pub fn push(
		&mut self,
		kind: Defense,
		damage_type: Option<Value<DamageType>>,
		context: Option<String>,
		source: PathBuf,
	) {
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
	pub max_height: (i32, RollSet),
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct MaxHitPoints(i32, BTreeMap<PathBuf, i32>);
impl MaxHitPoints {
	pub fn push(&mut self, bonus: i32, source: PathBuf) {
		self.0 = self.0.saturating_add(bonus);
		self.1.insert(source, bonus);
	}

	pub fn value(&self) -> u32 {
		self.0 as u32
	}

	pub fn sources(&self) -> &BTreeMap<PathBuf, i32> {
		&self.1
	}
}
