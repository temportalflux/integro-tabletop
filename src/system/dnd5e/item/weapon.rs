use crate::system::dnd5e::{roll::Roll, Ability};

#[derive(Clone, PartialEq)]
pub struct Weapon {
	pub kind: Kind,
	pub damage: Roll,
	pub damage_type: String,
	pub properties: Vec<Property>,
	pub range: Option<Range>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
	#[default]
	Simple,
	Martial,
}
#[derive(Clone, PartialEq)]
pub enum Property {
	Light,   // used by two handed fighting feature
	Finesse, // melee weapons use strength, ranged use dex, finesse take the better of either modifier
	Heavy,   // small or tiny get disadvantage on attack rolls when using this weapon
	Reach,
	TwoHanded,
	Thrown(u32, u32),
	Versatile(Roll),
}

#[derive(Clone, PartialEq, Default)]
pub struct Range {
	pub short_range: u32,
	pub long_range: u32,
	pub requires_ammunition: bool,
	pub requires_loading: bool,
}

#[derive(Clone, PartialEq, Default)]
pub struct Restriction {
	pub weapon_kind: Vec<Kind>,
	pub attack_kind: Vec<AttackType>,
	pub ability: Vec<Ability>,
}

#[derive(Clone, PartialEq)]
pub enum AttackType {
	Melee,
	Ranged,
}
