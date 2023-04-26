use crate::system::{
	core::SourceId,
	dnd5e::data::{action::LimitedUses, Ability},
};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Spellcasting {
	// Output goals:
	// - cantrip capacity
	// - cantrips prepared
	// - spell slot map (rank to slot capacity and number used)
	// - spell capacity (number of spells that can be prepared/known)
	// - spells prepared (or known)
	always_prepared: Vec<(SourceId, Ability, Option<LimitedUses>)>,
}

impl Spellcasting {
	pub fn add_prepared(
		&mut self,
		spell_ids: &Vec<SourceId>,
		ability: Ability,
		limited_uses: Option<&LimitedUses>,
	) {
		for spell_id in spell_ids {
			self.always_prepared
				.push((spell_id.clone(), ability, limited_uses.cloned()));
		}
	}

	pub fn prepared_spells(&self) -> &Vec<(SourceId, Ability, Option<LimitedUses>)> {
		&self.always_prepared
	}
}
