use crate::system::dnd5e::data::{
	action::LimitedUses,
	spell::{self, CastingDuration},
	Ability,
};
use std::path::PathBuf;

#[derive(Clone, PartialEq, Debug)]
pub enum CastingMethod {
	AtWill,
	Cast {
		can_use_slots: bool,
		can_use_ritual: bool,
	},
	LimitedUses(LimitedUses),
	FromContainer {
		item_id: Vec<uuid::Uuid>,
		consume_spell: bool,
		consume_item: bool,
	},
}

#[derive(Clone, PartialEq, Debug)]
pub struct SpellEntry {
	// About the origin
	pub source: PathBuf,
	pub classified_as: Option<String>,
	// How to Cast
	pub method: CastingMethod,
	pub attack_bonus: AbilityOrStat<i32>,
	pub save_dc: AbilityOrStat<u8>,
	pub damage_ability: Option<Ability>,
	// Overrides
	pub casting_duration: Option<CastingDuration>,
	pub rank: Option<u8>,
	pub range: Option<spell::Range>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AbilityOrStat<T> {
	Ability(Ability),
	Stat(T),
}
