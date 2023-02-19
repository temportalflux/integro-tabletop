use super::Mutator;
use crate::system::dnd5e::{character::DerivedBuilder, item::weapon, Value};

#[derive(Clone, PartialEq)]
pub struct BonusDamage {
	pub amount: Value<i32>,
	pub restriction: Option<weapon::Restriction>,
}
impl Mutator for BonusDamage {
	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		// TODO: For each equipped weapon, if the restriction is met, apply the bonus to the attack
	}
}
