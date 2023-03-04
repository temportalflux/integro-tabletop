use super::{Dependencies, TraitEq, AsTraitEq};
use downcast_rs::{impl_downcast, DowncastSync};
use std::{fmt::Debug, sync::Arc};

pub trait Evaluator: DowncastSync + Debug + TraitEq + AsTraitEq<dyn TraitEq> {
	type Context;
	type Item;

	/// The mutators this evaluator depends on.
	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn evaluate(&self, context: &Self::Context) -> Self::Item;
}
impl_downcast!(Evaluator assoc Context, Item);

pub type ArcEvaluator<C, V> = Arc<dyn Evaluator<Context = C, Item = V> + 'static + Send + Sync>;
#[derive(Clone)]
pub struct GenericEvaluator<C, V>(ArcEvaluator<C, V>);

impl<T, C, V> From<T> for GenericEvaluator<C, V>
where
	T: Evaluator<Context = C, Item = V> + 'static + Send + Sync,
{
	fn from(value: T) -> Self {
		Self(Arc::new(value))
	}
}

impl<C, V> GenericEvaluator<C, V> {
	pub fn new(value: ArcEvaluator<C, V>) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcEvaluator<C, V> {
		self.0
	}
}

impl<C, V> PartialEq for GenericEvaluator<C, V> where C: 'static, V: 'static {
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl<C, V> std::ops::Deref for GenericEvaluator<C, V> {
	type Target = ArcEvaluator<C, V>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<C, V> std::fmt::Debug for GenericEvaluator<C, V> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}
