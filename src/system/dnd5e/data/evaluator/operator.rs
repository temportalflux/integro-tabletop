use std::fmt::Debug;

use crate::{
	system::dnd5e::{data::character::Character, Value},
	utility::{Dependencies, Evaluator},
};

#[derive(Clone, PartialEq, Debug)]
pub struct Product<T>(pub Vec<Value<T>>);

impl<T> crate::utility::TraitEq for Product<T>
where
	T: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl<T> Evaluator for Product<T>
where
	T: 'static + std::iter::Product + Clone + Send + Sync + Debug + PartialEq,
{
	type Context = Character;
	type Item = T;

	fn dependencies(&self) -> Dependencies {
		self.0.iter().fold(Dependencies::default(), |deps, value| {
			deps.join(value.dependencies())
		})
	}

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		self.0.iter().map(|value| value.evaluate(state)).product()
	}
}
