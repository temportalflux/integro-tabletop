use crate::system::dnd5e::Ability;

#[derive(Clone, PartialEq)]
pub struct Armor {
	pub kind: Kind,
	/// The minimum armor-class granted while this is equipped.
	pub base_score: u32,
	/// The ability modifier granted to AC.
	pub ability_modifier: Option<Ability>,
	/// The maximum ability modifier granted. If none, the modifier is unbounded.
	pub max_ability_bonus: Option<i32>,
	/// The minimum expected strength score to use this armor.
	/// If provided, characters with a value less than this are hindered (reduced speed).
	pub min_strength_score: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
	Light,
	Medium,
	Heavy,
}
impl ToString for Kind {
	fn to_string(&self) -> String {
		format!("{self:?}")
	}
}
