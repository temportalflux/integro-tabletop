use crate::{
	system::dnd5e::{
		data::{action::Action, character::Character, item::weapon},
		Value,
	},
	utility::{Dependencies, Evaluator, Mutator},
};

#[derive(Clone, PartialEq)]
pub struct AddAction(pub Action);
impl Mutator for AddAction {
	type Target = Character;

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
	type Target = Character;

	fn node_id(&self) -> &'static str {
		"bonus_damage"
	}

	fn dependencies(&self) -> Dependencies {
		Dependencies::from(["add_action"]).join(self.amount.dependencies())
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		let bonus_amt = self.amount.evaluate(stats);
		for action in stats.iter_actions_mut_for(&self.restriction) {
			let Some(attack) = &mut action.attack else { continue; };
			let Some(damage) = &mut attack.damage else { continue; };
			damage.additional_bonuses.push((bonus_amt, source.clone()));
		}
	}
}
