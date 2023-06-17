use crate::{
	system::dnd5e::{data::character::Character, BoxedEvaluator},
	utility::{Dependencies, Evaluator},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Any(pub Vec<BoxedEvaluator<bool>>);

crate::impl_trait_eq!(Any);
crate::impl_kdl_node!(Any, "any");
impl Evaluator for Any {
	type Context = Character;
	type Item = bool;

	fn description(&self) -> Option<String> {
		None
	}

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
