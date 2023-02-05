use super::{Character, CompiledStats, Feature};
use crate::system::dnd5e::modifier;
use std::path::PathBuf;

#[derive(Default, Clone, PartialEq)]
pub struct Upbringing {
	pub name: String,
	pub description: String,
	pub features: Vec<Feature>,
}

impl Upbringing {
	pub fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}
}

impl modifier::Container for Upbringing {
	fn apply_modifiers(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		for feat in &self.features {
			feat.apply_modifiers(char, stats, scope.join(&feat.id()));
		}
	}
}
