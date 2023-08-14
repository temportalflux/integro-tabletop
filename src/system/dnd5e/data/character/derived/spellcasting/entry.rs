use crate::system::dnd5e::data::{action::LimitedUses, spell, Ability};
use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug)]
pub struct SpellEntry {
	pub ability: Ability,
	pub source: PathBuf,
	pub classified_as: Option<String>,
	pub cast_via_slot: bool,
	pub cast_via_ritual: bool,
	pub cast_via_uses: Option<LimitedUses>,
	pub range: Option<spell::Range>,
	pub forced_rank: Option<u8>,
}
