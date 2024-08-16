use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::data::{character::MAX_SPELL_RANK, Spell},
		SourceId,
	},
	utility::NotInList,
};
use itertools::Itertools;
use kdlize::{
	ext::{DocumentExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SpellCapability {
	Damage,
	Healing,
}

impl std::str::FromStr for SpellCapability {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Damage" => Ok(Self::Damage),
			"Healing" => Ok(Self::Healing),
			_ => Err(NotInList(s.to_owned(), vec!["Damage", "Healing"])),
		}
	}
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Filter {
	// If provided, the spell's must or must not be able to be cast as a ritual.
	pub ritual: Option<bool>,
	/// The spell must be of one of these ranks (or in the rank range).
	pub discrete_ranks: HashSet<u8>,
	pub rank_range: (Option<u8>, Option<u8>),
	/// The spell must have all of these tags,
	/// or be specified via `additional_tags`.
	pub tags: HashSet<String>,
	pub school_tag: Option<String>,
	// The spell must have the defined effect/feature/capability.
	pub capabilities: HashSet<SpellCapability>,
	/// Spells in this list can by-pass the `tags` requirement.
	pub additional_ids: HashSet<SourceId>,
}

impl FromKdl<NodeContext> for Filter {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut discrete_ranks = HashSet::new();
		let mut rank_range = (None, None);
		for mut node in node.query_all("scope() > rank")? {
			let entry = node.next_req()?;
			if entry.value().as_string() == Some("range") {
				let min = node.get_i64_opt("min")?.map(|v| v as u8);
				let max = node.get_i64_opt("max")?.map(|v| v as u8);
				rank_range = (min, max);
			} else {
				let rank = entry.as_i64_req()? as u8;
				discrete_ranks.insert(rank);
			}
		}

		let tags = node.query_str_all_t("scope() > tag", 0)?;
		let school_tag = node.query_str_opt("scope() > school", 0)?.map(str::to_owned);

		let capabilities = node.query_str_all_t("scope() > capability", 0)?;
		let additional_ids = node.query_str_all_t("scope() > spell", 0)?;

		Ok(Filter { discrete_ranks, rank_range, tags, school_tag, capabilities, additional_ids, ..Default::default() })
	}
}
impl AsKdl for Filter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.children(("rank", self.discrete_ranks.iter().sorted()));
		if self.rank_range.0.is_some() || self.rank_range.1.is_some() {
			let mut rank_node = NodeBuilder::default();
			rank_node.entry("range");
			rank_node.entry(("min", self.rank_range.0.map(|v| v as i64)));
			rank_node.entry(("min", self.rank_range.1.map(|v| v as i64)));
			node.child(("rank", rank_node));
		}
		node.child(("school", &self.school_tag));
		node.children(("tag", self.tags.iter().sorted()));
		node.children(("spell", self.additional_ids.iter().sorted()));
		node
	}
}

impl Filter {
	fn valid_ranks(&self) -> HashSet<u8> {
		let mut all_ranks = self.discrete_ranks.clone();
		if self.rank_range.0.is_some() || self.rank_range.1.is_some() {
			let min = self.rank_range.0.unwrap_or(0);
			let max = self.rank_range.1.unwrap_or(MAX_SPELL_RANK);
			all_ranks.extend(min..=max);
		}
		all_ranks
	}

	pub fn matches(&self, spell: &Spell) -> bool {
		let all_ranks = self.valid_ranks();
		if !all_ranks.is_empty() && !all_ranks.contains(&spell.rank) {
			return false;
		}

		if !self.additional_ids.is_empty() || !self.tags.is_empty() {
			let mut has_all_tags_or_additional_id = false;
			if !self.additional_ids.is_empty() {
				let minimal_id = spell.id.unversioned();
				if self.additional_ids.contains(&minimal_id) {
					has_all_tags_or_additional_id = true;
				}
			}
			if !has_all_tags_or_additional_id && !self.tags.is_empty() {
				// the spell must contain all of the tags
				has_all_tags_or_additional_id = true;
				if let Some(tag) = &self.school_tag {
					if spell.school_tag.as_ref() != Some(tag) {
						has_all_tags_or_additional_id = false;
					}
				}
				for tag in &self.tags {
					if !spell.tags.contains(tag) {
						// detected required tag missing from spell
						has_all_tags_or_additional_id = false;
						break;
					}
				}
			}
			if !has_all_tags_or_additional_id {
				return false;
			}
		}

		if let Some(ritual_flag) = &self.ritual {
			if spell.casting_time.ritual != *ritual_flag {
				return false;
			}
		}

		for capability in &self.capabilities {
			match capability {
				SpellCapability::Damage if spell.damage.is_none() => {
					return false;
				}
				SpellCapability::Healing if spell.healing.is_none() => {
					return false;
				}
				_ => {}
			}
		}

		true
	}

	pub fn as_criteria(&self) -> crate::database::Criteria {
		use crate::database::Criteria;
		let mut criteria = Vec::new();

		// Using the valid rank range for this filter, insert the rank criteria.
		let all_ranks = self.valid_ranks();
		if !all_ranks.is_empty() {
			// What this means:
			// There exists a root-level metadata property `rank`.
			// That `rank` property is a number which matches
			// any value in `rank_range` (the list of valid ranks for this filter).
			let rank_matches = all_ranks.into_iter().map(|rank| Criteria::exact(rank));
			let rank_is_one_of = Criteria::any(rank_matches);
			criteria.push(Criteria::contains_prop("rank", rank_is_one_of).into());
		}

		// Has tags or is otherwise specified
		criteria.push(Criteria::any({
			let mut criteria = Vec::with_capacity(2);
			if let Some(tag) = &self.school_tag {
				criteria.push(Criteria::contains_prop("school", Criteria::exact(tag.as_str())).into());
			}
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

		for capability in &self.capabilities {
			match capability {
				SpellCapability::Damage => {
					criteria.push(Criteria::contains_prop("damage", Criteria::exact(true)).into());
				}
				SpellCapability::Healing => {
					criteria.push(Criteria::contains_prop("healing", Criteria::exact(true)).into());
				}
			}
		}

		Criteria::All(criteria)
	}
}
