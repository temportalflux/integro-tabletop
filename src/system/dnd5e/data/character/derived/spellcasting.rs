use crate::system::{
	core::SourceId,
	dnd5e::data::{action::LimitedUses, Ability},
};
use multimap::MultiMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Spellcasting {
	// Output goals:
	// - cantrip capacity
	// - cantrips prepared
	// - spell slot map (rank to slot capacity and number used)
	// - spell capacity (number of spells that can be prepared/known)
	// - spells prepared (or known)
	always_prepared: MultiMap<SourceId, SpellEntry>,
}

impl Spellcasting {
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
