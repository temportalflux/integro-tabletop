use crate::system::dnd5e::data::{action::LimitedUses, spell, Ability};
use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug)]
pub struct SpellEntry {
	pub source: PathBuf,
	pub classified_as: Option<String>,

	pub cast_via_slot: bool,
	pub cast_via_ritual: bool,
	pub cast_via_uses: Option<LimitedUses>,

	pub attack_bonus: AbilityOrStat<i32>,
	pub save_dc: AbilityOrStat<u8>,
	pub damage_ability: Option<Ability>,
	pub rank: Option<u8>,
	pub range: Option<spell::Range>,
}
impl SpellEntry {
	pub fn cast_at_will(&self) -> bool {
		!self.cast_via_slot && !self.cast_via_ritual && self.cast_via_uses.is_none()
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum AbilityOrStat<T> {
	Ability(Ability),
	Stat(T),
}
