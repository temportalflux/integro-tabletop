use super::Dependencies;
use std::rc::Rc;

pub trait Evaluator {
	type Context;
	type Item;

	/// The mutators this evaluator depends on.
	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn evaluate(&self, context: &Self::Context) -> Self::Item;
}

#[derive(Clone)]
pub struct RcEvaluator<C, V>(Rc<dyn Evaluator<Context = C, Item = V> + 'static>);
impl<C, V> PartialEq for RcEvaluator<C, V> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}
impl<C, V> std::ops::Deref for RcEvaluator<C, V> {
	type Target = Rc<dyn Evaluator<Context = C, Item = V> + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T, C, V> From<T> for RcEvaluator<C, V>
where
	T: Evaluator<Context = C, Item = V> + 'static,
{
	fn from(value: T) -> Self {
		Self(Rc::new(value))
	}
}
