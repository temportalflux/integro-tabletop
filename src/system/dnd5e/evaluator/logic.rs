use super::{BoxedEvaluator, Evaluator};
use crate::system::dnd5e::character::Character;

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
