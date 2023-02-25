use super::data::character::Character;
use dyn_clone::{clone_trait_object, DynClone};

pub trait Mutator: DynClone {
	fn node_id(&self) -> &'static str;

	fn dependencies(&self) -> Option<Vec<&'static str>> {
		None
	}

	fn id(&self) -> Option<&str> {
		None
	}

	fn apply<'c>(&self, _: &mut Character) {}
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
	fn display_id(&self) -> bool {
		true
	}

	fn id(&self) -> Option<String> {
		None
	}

	fn apply_mutators<'c>(&self, stats: &mut Character);
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
