use crate::system::dnd5e::{
	data::{action::Action, character::Character, item::weapon},
	mutator::Mutator,
	Value,
};

#[derive(Clone, PartialEq)]
pub struct AddAction(pub Action);
impl Mutator for AddAction {
	fn node_id(&self) -> &'static str {
		"add_action"
	}

	fn apply<'c>(&self, stats: &mut Character) {
		stats.actions_mut().push(self.0.clone());
	}
}

#[derive(Clone, PartialEq)]
pub struct BonusDamage {
	pub amount: Value<i32>,
	pub restriction: Option<weapon::Restriction>,
}
impl Mutator for BonusDamage {
	fn node_id(&self) -> &'static str {
		"bonus_damage"
	}

	fn dependencies(&self) -> Option<Vec<&'static str>> {
		Some(vec!["add_action"])
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		let bonus_amt = self.amount.evaluate(stats);
		for action in stats.iter_actions_mut_for(&self.restriction) {
			let Some(attack) = &mut action.attack else { continue; };
			attack
				.damage_roll
				.additional_bonuses
				.push((bonus_amt, source.clone()));
		}
	}
}
