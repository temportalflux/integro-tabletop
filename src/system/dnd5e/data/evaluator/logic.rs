use crate::system::dnd5e::{
	data::character::Character,
	evaluator::{BoxedEvaluator, Evaluator},
};

#[derive(Clone, PartialEq)]
pub struct Any(pub Vec<BoxedEvaluator<bool>>);

impl Evaluator for Any {
	type Item = bool;

	fn evaluate(&self, state: &Character) -> Self::Item {
		for evaluator in &self.0 {
			if evaluator.evaluate(state) {
				return true;
			}
		}
		false
	}
}
