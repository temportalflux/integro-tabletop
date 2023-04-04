use super::Kind;
use crate::system::dnd5e::data::{action::AttackKind, Ability};
use std::collections::HashSet;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Restriction {
	pub weapon_kind: HashSet<Kind>,
	pub attack_kind: HashSet<AttackKind>,
	pub ability: HashSet<Ability>,
}
