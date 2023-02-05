use super::{
	character::CompiledStats,
	modifier::{self, Modifier},
	Action, Character,
};
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct Feature {
	pub name: String,
	pub description: String,
	pub action: Option<Action>,
	pub modifiers: Vec<Box<dyn Modifier + 'static>>,
}

impl Feature {
	pub fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}
}

impl PartialEq for Feature {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.description == other.description
			&& self.action == other.action
	}
}

impl modifier::Container for Feature {
	fn apply_modifiers(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		for modifier in &self.modifiers {
			modifier.apply(
				char,
				stats,
				match modifier.scope_id() {
					Some(id) => scope.join(id),
					None => scope.clone(),
				},
			);
		}
	}
}
