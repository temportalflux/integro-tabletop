use crate::system::{
	core::SourceId,
	dnd5e::{
		data::{
			action::LimitedUses,
			character::{Character, Persistent},
			spell, Ability,
		},
		BoxedEvaluator,
	},
};
use std::{
	collections::{BTreeMap, HashMap, HashSet},
	path::PathBuf,
};

mod cantrips;
pub use cantrips::*;
mod slots;
pub use slots::*;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Spellcasting {
	// Output goals:
	// - cantrip capacity
	// - cantrips prepared
	// - spell slot map (rank to slot capacity and number used)
	// - spell capacity (number of spells that can be prepared/known)
	// - spells prepared (or known)
	casters: HashMap<String, Caster>,
	always_prepared: HashMap<SourceId, HashMap<PathBuf, SpellEntry>>,
}

impl Spellcasting {
	pub fn add_caster(&mut self, caster: Caster) {
		self.casters.insert(caster.name().clone(), caster);
	}

	pub fn add_prepared(&mut self, spell_id: &SourceId, entry: SpellEntry) {
		if !self.always_prepared.contains_key(&spell_id) {
			self.always_prepared
				.insert(spell_id.clone(), HashMap::new());
		}
		self.always_prepared
			.get_mut(spell_id)
			.unwrap()
			.insert(entry.source.clone(), entry);
	}

	pub fn cantrip_capacity(&self, persistent: &Persistent) -> Vec<(usize, &Restriction)> {
		let mut total_capacity = Vec::new();
		for (_id, caster) in &self.casters {
			let value = caster.cantrip_capacity(persistent);
			if value > 0 {
				total_capacity.push((value, &caster.restriction));
			}
		}
		total_capacity
	}

	/// Returns the spell slots the character has to cast from.
	/// If there are multiple caster features, the spell slots are determined from multiclassing rules.
	pub fn spell_slots(&self, character: &Character) -> Option<BTreeMap<u8, usize>> {
		// https://www.dndbeyond.com/sources/basic-rules/customization-options#MulticlassSpellcaster
		lazy_static::lazy_static! {
			static ref MULTICLASS_SLOTS: BTreeMap<usize, BTreeMap<u8, usize>> = BTreeMap::from([
				( 1, [ (1, 2) ].into()),
				( 2, [ (1, 3) ].into()),
				( 3, [ (1, 4), (2, 2) ].into()),
				( 4, [ (1, 4), (2, 3) ].into()),
				( 5, [ (1, 4), (2, 3), (3, 2) ].into()),
				( 6, [ (1, 4), (2, 3), (3, 3) ].into()),
				( 7, [ (1, 4), (2, 3), (3, 3), (4, 1) ].into()),
				( 8, [ (1, 4), (2, 3), (3, 3), (4, 2) ].into()),
				( 9, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 1) ].into()),
				(10, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2) ].into()),
				(11, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1) ].into()),
				(12, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1) ].into()),
				(13, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1) ].into()),
				(14, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1) ].into()),
				(15, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1), (8, 1) ].into()),
				(16, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1), (8, 1) ].into()),
				(17, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 2), (6, 1), (7, 1), (8, 1), (9, 1) ].into()),
				(18, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 3), (6, 1), (7, 1), (8, 1), (9, 1) ].into()),
				(19, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 3), (6, 2), (7, 1), (8, 1), (9, 1) ].into()),
				(20, [ (1, 4), (2, 3), (3, 3), (4, 3), (5, 3), (6, 2), (7, 2), (8, 1), (9, 1) ].into()),
			]);
		}

		if self.casters.is_empty() {
			return None;
		}

		let (total_caster_level, slots_by_level) = if self.casters.len() == 1 {
			let (_id, caster) = self.casters.iter().next().unwrap();
			let current_level = character.level(Some(&caster.class_name));
			(current_level, &caster.slots.slots_capacity)
		} else {
			let mut levels = 0;
			for (_id, caster) in &self.casters {
				let current_level = character.level(Some(&caster.class_name));
				levels += match caster.slots.multiclass_half_caster {
					false => current_level,
					true => current_level / 2,
				};
			}
			(levels, &*MULTICLASS_SLOTS)
		};

		for (level, ranks) in slots_by_level.iter().rev() {
			if *level <= total_caster_level {
				return Some(ranks.clone());
			}
		}

		None
	}

	pub fn prepared_spells(&self) -> &HashMap<SourceId, HashMap<PathBuf, SpellEntry>> {
		&self.always_prepared
	}

	pub fn has_casters(&self) -> bool {
		!self.casters.is_empty()
	}

	pub fn get_caster(&self, id: &str) -> Option<&Caster> {
		self.casters.get(id)
	}

	pub fn iter_casters(&self) -> impl Iterator<Item = &Caster> {
		self.casters.iter().map(|(_id, caster)| caster)
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Caster {
	pub class_name: String,
	pub ability: Ability,
	pub restriction: Restriction,
	pub cantrip_capacity: Option<BTreeMap<usize, usize>>,
	pub slots: Slots,
	pub spell_capacity: SpellCapacity,
	pub spell_entry: SpellEntry,
	pub ritual_capability: Option<RitualCapability>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CasterKind {
	Known,
	Prepared,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RitualCapability {
	/// If true, the caster can ritually cast all spells which:
	/// 1. have the ritual tag
	/// 2. are classified as spells for this caster
	///    (spell has the class tag or was classified and is Always Prepared)
	/// 3. are available (e.g. all cleric spells, a wizard's spellbook)
	pub available_spells: bool,
	/// If true, the caster can ritually cast all spells which:
	/// 1. have the ritual tag
	/// 2. are classified as spells for this caster
	///    (spell has the class tag or was classified and is Always Prepared)
	/// 3. are selected (i.e. prepared or known)
	pub selected_spells: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SpellCapacity {
	// the number of spells that can be known, keyed by class level
	Known(BTreeMap<usize, usize>),
	// the number of spells that can be prepared
	Prepared(BoxedEvaluator<i32>),
}

impl Caster {
	pub fn name(&self) -> &String {
		&self.class_name
	}

	pub fn kind(&self) -> CasterKind {
		match &self.spell_capacity {
			SpellCapacity::Known(_) => CasterKind::Known,
			SpellCapacity::Prepared(_) => CasterKind::Prepared,
		}
	}

	pub fn cantrip_capacity(&self, persistent: &Persistent) -> usize {
		let Some(capacity) = &self.cantrip_capacity else { return 0; };
		let current_level = persistent.level(Some(&self.class_name));
		for (level, count) in capacity.iter().rev() {
			if *level <= current_level {
				return *count;
			}
		}
		0
	}

	pub fn spell_capacity(&self, character: &Character) -> usize {
		match &self.spell_capacity {
			SpellCapacity::Known(capacity) => {
				let current_level = character.level(Some(&self.class_name));
				let mut max_amt = 0;
				for (level, amount) in capacity {
					if *level > current_level {
						break;
					}
					max_amt = *amount;
				}
				max_amt
			}
			SpellCapacity::Prepared(capacity) => capacity.evaluate(&character) as usize,
		}
	}

	/// Use to determine what kind of spells can be prepared/known.
	pub fn max_spell_rank(&self, character: &Character) -> Option<u8> {
		let current_level = character.level(Some(&self.class_name));
		let mut max_rank = None;
		for (level, rank_to_count) in &self.slots.slots_capacity {
			if *level > current_level {
				break;
			}
			max_rank = rank_to_count.keys().max().cloned();
		}
		max_rank
	}

	pub fn spell_filter(&self, character: &Character) -> SpellFilter {
		SpellFilter {
			can_cast: Some(self.name().clone()),
			//tags: caster.restriction.tags.iter().cloned().collect(),
			max_rank: self.max_spell_rank(character),
			..Default::default()
		}
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct SpellEntry {
	pub ability: Ability,
	pub source: PathBuf,
	pub classified_as: Option<String>,
	pub cast_via_slot: bool,
	pub cast_via_uses: Option<LimitedUses>,
	pub range: Option<spell::Range>,
	pub forced_rank: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpellFilter {
	/// The spell must already be castable by the provided caster class.
	/// This can be true if the spell contains the class tag OR the spell is in the expanded list
	/// for the caster data (e.g. spellcasting "add_source").
	pub can_cast: Option<String>,
	// If provided, the spell's must or must not be able to be cast as a ritual.
	pub ritual: Option<bool>,
	/// The spell must be of one of these ranks.
	pub ranks: HashSet<u8>,
	/// The spell's rank must be <= this rank.
	pub max_rank: Option<u8>,
	/// The spell must have all of these tags.
	pub tags: HashSet<String>,
}
impl SpellFilter {
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

		if !self.tags.is_empty() {
			// What this means:
			// There exists a root-level metadata property named `tags`.
			// That `tags` property is an array, which contains every value contained in the `self.tags` list.
			let tag_matches = self.tags.iter().map(|tag| Criteria::exact(tag.as_str()));
			let contains_match = tag_matches.map(|matcher| Criteria::contains_element(matcher));
			criteria.push(Criteria::contains_prop("tags", Criteria::all(contains_match)).into());
		}

		if let Some(caster_class) = &self.can_cast {
			// What this means:
			// Firstly, there exists a root-level metadata property named `tags`.
			// That `tags` property contains a string whose value matches the name of the caster class this filter is looking for.
			let has_class_tag = Criteria::contains_element(Criteria::exact(caster_class.as_str()));
			// TODO: check if the spell is in the expanded spell list,
			// as provided by the AddSource spellcasting mutator.
			let can_cast = Criteria::any(vec![has_class_tag.into()]);
			criteria.push(Criteria::contains_prop("tags", can_cast).into());
		}

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

	pub fn spell_matches(&self, spell: &spell::Spell) -> bool {
		if let Some(ritual_flag) = &self.ritual {
			if spell.casting_time.ritual != *ritual_flag {
				return false;
			}
		}
		if let Some(range) = self.rank_range::<HashSet<_>>() {
			if !range.contains(&spell.rank) {
				return false;
			}
		}
		if !self.tags.is_empty() {
			for tag in &self.tags {
				if !spell.tags.contains(tag) {
					return false;
				}
			}
		}
		if let Some(caster_class) = &self.can_cast {
			// TODO: check if the spell is in the expanded spell list,
			// as provided by the AddSource spellcasting mutator.
			if !spell.tags.contains(caster_class) {
				return false;
			}
		}
		true
	}
}
