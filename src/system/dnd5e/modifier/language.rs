use super::Selector;
use crate::system::dnd5e::{character::CompiledStats, Character};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AddLanguage(pub Selector<String>);

impl super::Modifier for AddLanguage {
	fn scope_id(&self) -> Option<&str> {
		self.0.id()
	}

	fn apply(&self, char: &Character, stats: &mut CompiledStats, scope: PathBuf) {
		let language = match &self.0 {
			Selector::Specific(language) => Some(language.clone()),
			_ => char.get_selection(stats, &scope).map(str::to_owned),
		};
		if let Some(lang) = language {
			stats.languages.push(lang);
		}
	}
}
