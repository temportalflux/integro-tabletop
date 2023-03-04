use crate::{
	system::dnd5e::{
		data::{action::Action, character::Character, item::weapon},
		KDLNode, Value,
	},
	utility::{Dependencies, Evaluator, Mutator},
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddAction(pub Action);
impl KDLNode for AddAction {
	fn id() -> &'static str {
		"add_action"
	}
}
impl Mutator for AddAction {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn apply<'c>(&self, stats: &mut Character) {
		stats.actions_mut().push(self.0.clone());
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct BonusDamage {
	pub amount: Value<i32>,
	pub restriction: Option<weapon::Restriction>,
}
impl KDLNode for BonusDamage {
	fn id() -> &'static str {
		"bonus_damage"
	}
}
impl Mutator for BonusDamage {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
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
