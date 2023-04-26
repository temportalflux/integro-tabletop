use crate::{
	system::dnd5e::{
		data::{character::Character, description, item::weapon},
		Value,
	},
	utility::{Dependencies, Evaluator, Mutator},
};

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
		// This technically depends on actions existing,
		// but since actions are features, and features are always
		// added before mutators are processed,
		// we don't have a specific mutator to call out here.
		self.amount.dependencies()
	}

	fn description(&self) -> description::Section {
		description::Section::default()
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
