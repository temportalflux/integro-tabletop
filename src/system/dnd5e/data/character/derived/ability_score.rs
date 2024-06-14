use crate::system::{
	dnd5e::data::{character::Character, description, Ability, Score},
	mutator::ReferencePath,
};
use enum_map::EnumMap;
use itertools::{Either, Itertools};
use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AbilityScores(EnumMap<Ability, AbilityScore>);
impl AbilityScores {
	pub fn push_bonus(&mut self, ability: Ability, bonus: AbilityScoreBonus, source: PathBuf) {
		self.0[ability].bonuses.push((bonus, source, false));
	}

	pub fn increase_maximum(&mut self, ability: Ability, max: u32, source: PathBuf) {
		self.0[ability].max_score_incs.push((max, source));
	}

	pub fn finalize(&mut self) {
		for ability_score in self.0.values_mut() {
			ability_score.finalize();
		}
	}

	pub fn get(&self, ability: Ability) -> &AbilityScore {
		&self[ability]
	}
}
impl std::ops::Index<Ability> for AbilityScores {
	type Output = AbilityScore;
	fn index(&self, index: Ability) -> &Self::Output {
		&self.0[index]
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct FinalizeAbilityScores;
crate::impl_trait_eq!(FinalizeAbilityScores);
kdlize::impl_kdl_node!(FinalizeAbilityScores, "ability_score_finalize");
impl crate::system::Mutator for FinalizeAbilityScores {
	type Target = Character;

	fn dependencies(&self) -> crate::utility::Dependencies {
		["ability_score"].into()
	}

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section::default()
	}

	fn apply(&self, stats: &mut Self::Target, _parent: &ReferencePath) {
		stats.ability_scores_mut().finalize();
	}
}
impl crate::kdl_ext::AsKdl for FinalizeAbilityScores {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		// STUB: Not registered for documents
		crate::kdl_ext::NodeBuilder::default()
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct AbilityScore {
	bonuses: Vec<(AbilityScoreBonus, PathBuf, /*was included*/ bool)>,
	max_score_incs: Vec<(u32, PathBuf)>,
	total: Score,
}
impl Default for AbilityScore {
	fn default() -> Self {
		Self {
			bonuses: vec![],
			max_score_incs: vec![(20, "Default Maximum".into())],
			total: Score(0),
		}
	}
}
impl AbilityScore {
	pub fn score(&self) -> Score {
		self.total
	}

	pub fn iter_bonuses(&self) -> impl Iterator<Item = &(AbilityScoreBonus, PathBuf, bool)> {
		self.bonuses.iter()
	}

	pub fn iter_max_increases(&self) -> impl Iterator<Item = &(u32, PathBuf)> {
		self.max_score_incs.iter()
	}

	pub fn finalize(&mut self) {
		let (max_possible_score, used_bonus_indices) = self.evaluate();
		*self.total = max_possible_score.min(self.eval_max_score());
		for (idx, (_bonus, _path, was_used)) in self.bonuses.iter_mut().enumerate() {
			*was_used = used_bonus_indices.contains(&idx);
		}
	}

	fn eval_max_score(&self) -> u32 {
		self.max_score_incs.iter().map(|(v, _)| *v).max().unwrap_or(0)
	}

	/// Returns the evaluated total scoree, and the list of paths that were used from the list of bonuses.
	fn evaluate(&self) -> (u32, HashSet<usize>) {
		let (no_constraints, constrained): (Vec<_>, Vec<_>) =
			self.bonuses
				.iter()
				.enumerate()
				.partition_map(|(idx, (bonus, _, _))| match bonus.max_total {
					None => Either::Left((idx, bonus.value)),
					Some(max_total) => Either::Right((idx, (bonus.value, max_total))),
				});
		let (mut used_indices, unconstrained): (HashSet<_>, Vec<_>) = no_constraints.into_iter().unzip();
		let total = unconstrained.into_iter().sum::<u32>();
		let (total, additional_indices) = optimize_max_sums(total, constrained);
		if let Some(indices) = additional_indices {
			used_indices.extend(indices.into_iter());
		}
		(total, used_indices)
	}
}
#[cfg(test)]
impl AbilityScore {
	pub fn with_total(mut self, max: u32) -> Self {
		self.total = Score(max);
		self
	}

	pub fn with_bonus(mut self, bonus: (u32, Option<u32>, PathBuf, bool)) -> Self {
		self.bonuses.push((
			AbilityScoreBonus {
				value: bonus.0,
				max_total: bonus.1,
			},
			bonus.2,
			bonus.3,
		));
		self
	}

	pub fn with_max_inc(mut self, max: (u32, PathBuf)) -> Self {
		self.max_score_incs.push(max);
		self
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AbilityScoreBonus {
	/// The value to add to the total score.
	pub value: u32,
	/// If provided, the value can only be applied if the resulting total is <= this constraint.
	pub max_total: Option<u32>,
}
impl From<u32> for AbilityScoreBonus {
	fn from(value: u32) -> Self {
		Self {
			value,
			..Default::default()
		}
	}
}

fn optimize_max_sums(
	base: u32,
	mut bonuses: Vec<(/*idx*/ usize, (/*bonus*/ u32, /*max*/ u32))>,
) -> (u32, Option<Vec<usize>>) {
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
			vec![(0, (1, 17)), (1, (2, 17)), (2, (3, 17)), (3, (4, 17)), (4, (5, 17))],
		);
		assert_eq!(value, (17, Some(vec![3, 4])));
	}
}
