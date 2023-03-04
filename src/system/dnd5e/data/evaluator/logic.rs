use crate::{
	system::dnd5e::{data::character::Character, BoxedEvaluator},
	utility::{Dependencies, Evaluator},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Any(pub Vec<BoxedEvaluator<bool>>);

impl crate::utility::TraitEq for Any {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl Evaluator for Any {
	type Context = Character;
	type Item = bool;

	fn dependencies(&self) -> Dependencies {
		self.0.iter().fold(Dependencies::default(), |deps, eval| {
			deps.join(eval.dependencies())
		})
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		for evaluator in &self.0 {
			if evaluator.evaluate(state) {
				return true;
			}
		}
		false
	}
}
