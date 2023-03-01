use super::Dependencies;
use std::rc::Rc;

pub trait Mutator {
	type Target;

	fn node_id(&self) -> &'static str;

	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, _: &mut Self::Target) {}
}

#[derive(Clone)]
pub struct RcMutator<T>(Rc<dyn Mutator<Target = T> + 'static>);
impl<T> PartialEq for RcMutator<T> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}
impl<T> std::ops::Deref for RcMutator<T> {
	type Target = Rc<dyn Mutator<Target = T> + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<M, T> From<M> for RcMutator<T>
where
	M: Mutator<Target = T> + 'static,
{
	fn from(value: M) -> Self {
		Self(Rc::new(value))
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
