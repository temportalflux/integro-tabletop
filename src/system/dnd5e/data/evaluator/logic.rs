use crate::{
	system::dnd5e::{data::character::Character, BoxedEvaluator},
	utility::Evaluator,
};

#[derive(Clone, PartialEq)]
pub struct Any(pub Vec<BoxedEvaluator<bool>>);

impl Evaluator for Any {
	type Context = Character;
	type Item = bool;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		for evaluator in &self.0 {
			if evaluator.evaluate(state) {
				return true;
			}
		}
		false
	}
}
