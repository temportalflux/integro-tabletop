use super::{AsTraitEq, Dependencies, TraitEq};
use crate::system::dnd5e::KDLNode;
use std::{fmt::Debug, sync::Arc};

pub trait Mutator: Debug + TraitEq + AsTraitEq<dyn TraitEq> + KDLNode {
	type Target;

	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn data_id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, _: &mut Self::Target) {}
}

pub type ArcMutator<T> = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;
#[derive(Clone)]
pub struct GenericMutator<T>(ArcMutator<T>);

impl<M, T> From<M> for GenericMutator<T>
where
	M: Mutator<Target = T> + 'static + Send + Sync,
{
	fn from(value: M) -> Self {
		Self(Arc::new(value))
	}
}

impl<T> GenericMutator<T> {
	pub fn new(value: ArcMutator<T>) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcMutator<T> {
		self.0
	}
}

impl<T> PartialEq for GenericMutator<T>
where
	T: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl<T> std::ops::Deref for GenericMutator<T> {
	type Target = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> std::fmt::Debug for GenericMutator<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

pub trait MutatorGroup {
	type Target;

	fn display_id(&self) -> bool {
		true
	}

	fn id(&self) -> Option<String> {
		None
	}

	fn apply_mutators<'c>(&self, target: &mut Self::Target);
}
