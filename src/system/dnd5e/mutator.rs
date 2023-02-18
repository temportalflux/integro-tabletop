use super::character::DerivedBuilder;
use dyn_clone::{clone_trait_object, DynClone};

mod ability_score;
pub use ability_score::*;

mod description;
pub use description::*;

mod defense;
pub use defense::*;

mod language;
pub use language::*;

mod saving_throw;
pub use saving_throw::*;

mod skill;
pub use skill::*;

mod speed;
pub use speed::*;

pub trait Mutator: DynClone {
	fn scope_id(&self) -> Option<&str> {
		None
	}
	fn apply<'c>(&self, _: &mut DerivedBuilder<'c>) {}
}
clone_trait_object!(Mutator);

#[derive(Clone)]
pub struct BoxedMutator(std::rc::Rc<dyn Mutator + 'static>);
impl PartialEq for BoxedMutator {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for BoxedMutator {
	type Target = std::rc::Rc<dyn Mutator + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T> From<T> for BoxedMutator
where
	T: Mutator + 'static,
{
	fn from(value: T) -> Self {
		Self(std::rc::Rc::new(value))
	}
}

pub trait Container {
	fn id(&self) -> Option<String> {
		None
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>);
}

#[derive(Clone)]
pub enum Selector<T> {
	Specific(T),
	AnyOf { id: Option<String>, options: Vec<T> },
	Any { id: Option<String> },
}

impl<T> Selector<T> {
	pub fn id(&self) -> Option<&str> {
		match self {
			Self::Specific(_) => None,
			Self::AnyOf { id, options: _ } => id.as_ref(),
			Self::Any { id } => id.as_ref(),
		}
		.map(String::as_str)
	}
}
