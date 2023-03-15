use crate::{
	system::dnd5e::{
		data::{action::Action, character::Character, item::weapon},
		Value,
	},
	utility::{Dependencies, Evaluator, Mutator},
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddAction(pub Action);
crate::impl_trait_eq!(AddAction);
crate::impl_kdl_node!(AddAction, "add_action");
impl Mutator for AddAction {
	type Target = Character;

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		stats.actions_mut().push(self.0.clone());
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct BonusDamage {
	pub amount: Value<i32>,
	pub restriction: Option<weapon::Restriction>,
}
crate::impl_trait_eq!(BonusDamage);
crate::impl_kdl_node!(BonusDamage, "bonus_damage");
impl Mutator for BonusDamage {
	type Target = Character;

	fn dependencies(&self) -> Dependencies {
		Dependencies::from(["add_action"]).join(self.amount.dependencies())
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		let bonus_amt = self.amount.evaluate(stats);
		for action in stats.iter_actions_mut_for(&self.restriction) {
			let Some(attack) = &mut action.attack else { continue; };
			let Some(damage) = &mut attack.damage else { continue; };
			damage
				.additional_bonuses
				.push((bonus_amt, parent.to_owned()));
		}
	}
}
