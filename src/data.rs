use yew::prelude::*;

/// A wrapper for a yew reducer state through
/// which the underlying data can be mutated by components.
#[derive(Clone, PartialEq)]
pub struct ContextMut<T: Reducible>(UseReducerHandle<T>);

impl<T> From<UseReducerHandle<T>> for ContextMut<T>
where
	T: Reducible,
{
	fn from(value: UseReducerHandle<T>) -> Self {
		Self(value)
	}
}

impl<T> ContextMut<T>
where
	T: Reducible,
{
	pub fn dispatch(&self, value: T::Action) {
		self.0.dispatch(value);
	}
}

impl<T> ContextMut<T>
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

	pub fn new_mutator<F, I>(&self, callback: F) -> Callback<I>
	where
		T: 'static,
		F: Fn(&mut T) + 'static,
		I: 'static,
	{
		let ctx = self.0.clone();
		let mutator = std::rc::Rc::new(callback);
		Callback::from(move |_: I| {
			let mutator = mutator.clone();
			ctx.dispatch(Callback::from(move |mut data| {
				(*mutator)(&mut data);
				data
			}))
		})
	}
}

impl<T> std::ops::Deref for ContextMut<T>
where
	T: Reducible,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&*self.0
	}
}
