use crate::system::dnd5e::character::DerivedBuilder;

use super::Selector;

#[derive(Clone)]
pub struct AddLanguage(pub Selector<String>);

impl super::Mutator for AddLanguage {
	fn scope_id(&self) -> Option<&str> {
		self.0.id()
	}

	fn apply<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		let language = match &self.0 {
			Selector::Specific(language) => Some(language.clone()),
			_ => stats.get_selection().map(str::to_owned),
		};
		if let Some(lang) = language {
			stats.add_language(lang);
		}
	}
}
