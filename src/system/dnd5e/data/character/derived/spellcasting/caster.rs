use super::{Filter, Slots, SpellEntry};
use crate::system::dnd5e::{
	data::{
		character::{Character, Persistent},
		Ability,
	},
	BoxedEvaluator,
};
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Debug)]
pub struct Caster {
	pub class_name: String,
	pub ability: Ability,
	pub restriction: Restriction,
	pub cantrip_capacity: Option<BTreeMap<usize, usize>>,
	pub slots: Slots,
	pub spell_capacity: Capacity,
	pub spell_entry: SpellEntry,
	pub ritual_capability: Option<RitualCapability>,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Restriction {
	pub tags: Vec<String>,
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
pub enum Capacity {
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
			Capacity::Known(_) => CasterKind::Known,
			Capacity::Prepared(_) => CasterKind::Prepared,
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
			Capacity::Known(capacity) => {
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
			Capacity::Prepared(capacity) => capacity.evaluate(&character) as usize,
		}
	}

	/// Use to determine what kind of spells can be prepared/known.
	pub fn max_spell_rank(&self, persistent: &Persistent) -> Option<u8> {
		let current_level = persistent.level(Some(&self.class_name));
		let mut max_rank = None;
		for (level, rank_to_count) in &self.slots.slots_capacity {
			if *level > current_level {
				break;
			}
			max_rank = rank_to_count.keys().max().cloned();
		}
		max_rank
	}

	pub fn spell_filter(&self, persistent: &Persistent) -> Filter {
		Filter {
			can_cast: Some(self.name().clone()),
			//tags: caster.restriction.tags.iter().cloned().collect(),
			max_rank: self.max_spell_rank(persistent),
			..Default::default()
		}
	}
}
