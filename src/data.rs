use yew::prelude::*;

pub use crate::system::dnd5e::{Ability, ProficiencyLevel, Skill};

/// A wrapper for a yew reducer state through
/// which the underlying data can be mutated by components.
#[derive(Clone, PartialEq)]
pub struct Context<T: Reducible>(UseReducerHandle<T>);

impl<T> From<UseReducerHandle<T>> for Context<T>
where
	T: Reducible,
{
	fn from(value: UseReducerHandle<T>) -> Self {
		Self(value)
	}
}

impl<T> Context<T>
where
	T: Reducible,
{
	pub fn dispatch(&self, value: T::Action) {
		self.0.dispatch(value);
	}
}

impl<T> Context<T>
where
	T: Reducible<Action = Callback<T, T>>,
{
	pub fn mutate<F>(&self, callback: F)
	where
		F: Fn(&mut T) + 'static,
	{
		self.dispatch(Callback::from(move |mut data| {
			callback(&mut data);
			data
		}));
	}
}

impl<T> std::ops::Deref for Context<T>
where
	T: Reducible,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}
