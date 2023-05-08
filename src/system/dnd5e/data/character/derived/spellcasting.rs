use crate::system::{
	core::SourceId,
	dnd5e::{
		data::{
			action::LimitedUses,
			character::{Character, Persistent},
			Ability,
		},
		BoxedEvaluator,
	},
};
use multimap::MultiMap;
use std::{
	collections::{BTreeMap, HashMap},
	path::{Path, PathBuf},
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
	always_prepared: MultiMap<SourceId, SpellEntry>,
}

impl Spellcasting {
	pub fn add_caster(&mut self, caster: Caster) {
		self.casters.insert(caster.name().clone(), caster);
	}

	pub fn add_prepared(
		&mut self,
		spell_ids: &Vec<SourceId>,
		ability: Ability,
		limited_uses: Option<&LimitedUses>,
		source: impl AsRef<Path>,
	) {
		for spell_id in spell_ids {
			let entry = SpellEntry {
				ability,
				capability: match limited_uses {
					Some(uses) => SpellCapability::LimitedUses(uses.clone()),
					None => SpellCapability::Prepared,
				},
				source: source.as_ref().to_owned(),
			};
			self.always_prepared.insert(spell_id.clone(), entry);
		}
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

	pub fn prepared_spells(&self) -> &MultiMap<SourceId, SpellEntry> {
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
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CasterKind {
	Known,
	Prepared,
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
}

#[derive(Clone, PartialEq, Debug)]
pub struct SpellEntry {
	pub ability: Ability,
	pub capability: SpellCapability,
	pub source: PathBuf,
}

#[derive(Clone, PartialEq, Debug)]
pub enum SpellCapability {
	Prepared,
	LimitedUses(LimitedUses),
}
