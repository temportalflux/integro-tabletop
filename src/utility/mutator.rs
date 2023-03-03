use super::Dependencies;
use std::sync::Arc;

pub trait Mutator {
	type Target;

	fn node_name() -> &'static str
	where
		Self: Sized;

	fn get_node_name(&self) -> &'static str;

	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, _: &mut Self::Target) {}
}

#[derive(Clone)]
pub struct ArcMutator<T>(Arc<dyn Mutator<Target = T> + 'static + Send + Sync>);
impl<T> PartialEq for ArcMutator<T> {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}
impl<T> std::ops::Deref for ArcMutator<T> {
	type Target = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<M, T> From<M> for ArcMutator<T>
where
	M: Mutator<Target = T> + 'static + Send + Sync,
{
	fn from(value: M) -> Self {
		Self(Arc::new(value))
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
