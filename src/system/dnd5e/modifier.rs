use super::character::StatsBuilder;
use dyn_clone::{clone_trait_object, DynClone};

mod ability_score;
pub use ability_score::*;

mod description;
pub use description::*;

mod skill;
pub use skill::*;

mod language;
pub use language::*;

pub trait Modifier: DynClone {
	fn scope_id(&self) -> Option<&str> {
		None
	}
	fn apply<'c>(&self, _: &mut StatsBuilder<'c>) {}
}
clone_trait_object!(Modifier);

#[derive(Clone)]
pub struct BoxedModifier(std::rc::Rc<dyn Modifier + 'static>);
impl PartialEq for BoxedModifier {
	fn eq(&self, other: &Self) -> bool {
		std::rc::Rc::ptr_eq(&self.0, &other.0)
	}
}
impl std::ops::Deref for BoxedModifier {
	type Target = std::rc::Rc<dyn Modifier + 'static>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T> From<T> for BoxedModifier
where
	T: Modifier + 'static,
{
	fn from(value: T) -> Self {
		Self(std::rc::Rc::new(value))
	}
}

pub trait Container {
	fn id(&self) -> String;
	fn apply_modifiers<'c>(&self, stats: &mut StatsBuilder<'c>);
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
