use crate::system::{
	core::SourceId,
	dnd5e::data::{action::LimitedUses, character::Persistent, Ability},
};
use multimap::MultiMap;
use std::path::{Path, PathBuf};

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
	cantrips: Vec<CantripCapacity>,
	always_prepared: MultiMap<SourceId, SpellEntry>,
}

impl Spellcasting {
	pub fn add_cantrip_capacity(&mut self, capacity: CantripCapacity) {
		self.cantrips.push(capacity);
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
		for capacity in &self.cantrips {
			let current_level = persistent.level(Some(&capacity.class_name));
			let value = {
				let mut max_count = 0;
				for (level, count) in &capacity.level_map {
					if *level > current_level {
						break;
					}
					max_count = *count;
				}
				max_count
			};
			total_capacity.push((value, &capacity.restriction));
		}
		total_capacity
	}

	pub fn prepared_spells(&self) -> &MultiMap<SourceId, SpellEntry> {
		&self.always_prepared
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
