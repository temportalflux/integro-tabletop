use super::character::{Character, CompiledStats};
use std::path::PathBuf;

mod ability_score;
pub use ability_score::*;

mod description;
pub use description::*;

mod skill;
pub use skill::*;

mod language;
pub use language::*;

pub trait Modifier: BoxedClone {
	fn scope_id(&self) -> Option<&str> {
		None
	}
	fn apply(&self, _: &Character, _: &mut CompiledStats, _: PathBuf) {}
}
pub trait BoxedClone {
	fn clone_box<'a>(&self) -> Box<dyn Modifier>;
}
impl<T> BoxedClone for T
where
	T: Modifier + Clone + 'static,
{
	fn clone_box<'a>(&self) -> Box<dyn Modifier> {
		Box::new(self.clone())
	}
}
impl Clone for Box<dyn Modifier> {
	fn clone(&self) -> Box<dyn Modifier> {
		self.clone_box()
	}
}

pub trait Container {
	fn apply_modifiers(&self, character: &Character, stats: &mut CompiledStats, scope: PathBuf);
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
