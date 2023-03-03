use super::Dependencies;
use std::sync::Arc;

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
pub struct RcEvaluator<C, V>(Arc<dyn Evaluator<Context = C, Item = V> + 'static + Send + Sync>);
impl<C, V> PartialEq for RcEvaluator<C, V> {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}
impl<C, V> std::ops::Deref for RcEvaluator<C, V> {
	type Target = Arc<dyn Evaluator<Context = C, Item = V> + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T, C, V> From<T> for RcEvaluator<C, V>
where
	T: Evaluator<Context = C, Item = V> + 'static + Send + Sync,
{
	fn from(value: T) -> Self {
		Self(Arc::new(value))
	}
}
