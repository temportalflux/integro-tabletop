use crate::{system::dnd5e::data::character::Character, utility::Evaluator};
use std::fmt::Debug;

/// Returns the numerical value of the level for a character.
/// Optionally can return the level for a specific class, if `class_name` is specified.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct GetLevel<T> {
	class_name: Option<String>,
	marker: std::marker::PhantomData<T>,
}
impl<T, S> From<Option<S>> for GetLevel<T>
where
	S: ToString,
	T: Default,
{
	fn from(value: Option<S>) -> Self {
		Self {
			class_name: value.map(|s| s.to_string()),
			marker: std::marker::PhantomData::default(),
		}
	}
}

impl<T> crate::utility::TraitEq for GetLevel<T>
where
	T: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl<T> Evaluator for GetLevel<T>
where
	T: 'static + Copy + Debug + Send + Sync + PartialEq,
	usize: num_traits::AsPrimitive<T>,
{
	type Context = Character;
	type Item = T;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		use num_traits::AsPrimitive;
		let value = state
			.level(self.class_name.as_ref().map(String::as_str))
			.as_();
		value
	}
}
