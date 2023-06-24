use crate::system::core::SourceId;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Filter {
	// If provided, the spell's must or must not be able to be cast as a ritual.
	pub ritual: Option<bool>,
	/// The spell must be of one of these ranks.
	pub ranks: HashSet<u8>,
	/// The spell's rank must be <= this rank.
	pub max_rank: Option<u8>,

	/// The spell must have all of these tags,
	/// or be specified via `additional_tags`.
	pub tags: HashSet<String>,
	/// Spells in this list can by-pass the `tags` requirement.
	pub additional_ids: HashSet<SourceId>,
}

impl Filter {
	fn rank_range<T>(&self) -> Option<T>
	where
		T: FromIterator<u8>,
	{
		match self.max_rank {
			Some(max_rank) => Some((0..=max_rank).collect::<T>()),
			None if !self.ranks.is_empty() => Some(self.ranks.iter().map(|i| *i).collect::<T>()),
			None => None,
		}
	}

	pub fn as_criteria(&self) -> crate::database::app::Criteria {
		use crate::database::app::Criteria;
		let mut criteria = Vec::new();

		// Using the valid rank range for this filter, insert the rank criteria.
		// The valid rank range is derived from `self.max_rank` and `self.ranks`.
		if let Some(rank_range) = self.rank_range::<Vec<_>>() {
			// What this means:
			// There exists a root-level metadata property `rank`.
			// That `rank` property is a number which matches
			// any value in `rank_range` (the list of valid ranks for this filter).
			let rank_matches = rank_range.into_iter().map(|rank| Criteria::exact(rank));
			let rank_is_one_of = Criteria::any(rank_matches);
			criteria.push(Criteria::contains_prop("rank", rank_is_one_of).into());
		}

		// Has tags or is otherwise specified
		criteria.push(Criteria::any({
			let mut criteria = Vec::with_capacity(2);
			if !self.tags.is_empty() {
				// What this means:
				// There exists a root-level metadata property named `tags`.
				// That `tags` property is an array, which contains every value contained in the `self.tags` list.
				let tag_matches = self.tags.iter().map(|tag| Criteria::exact(tag.as_str()));
				let contains_match = tag_matches.map(|matcher| Criteria::contains_element(matcher));
				let all_tags_match = Criteria::all(contains_match);
				criteria.push(Criteria::contains_prop("tags", all_tags_match).into());
			}
			if !self.additional_ids.is_empty() {
				for id in &self.additional_ids {
					let matches_id = Criteria::exact(id.to_string());
					criteria.push(Criteria::contains_prop("id", matches_id));
				}
			}
			criteria
		}));

		if let Some(ritual_flag) = &self.ritual {
			// What this means:
			// There exists a root-level metadata property named `casting`.
			// The `casting` property is an object which has a property named `ritual`.
			// The value of that `ritual` property is a (json) boolean
			// with a value which matches the provided `ritual` flag.
			let matches_ritual = Criteria::exact(*ritual_flag);
			let ritual = Criteria::contains_prop("ritual", matches_ritual);
			let casting = Criteria::contains_prop("casting", ritual);
			criteria.push(casting.into());
		}

		Criteria::All(criteria)
	}
}
