use crate::system::dnd5e::character::StatsBuilder;

use super::Selector;

#[derive(Clone)]
pub struct AddLanguage(pub Selector<String>);

impl super::Modifier for AddLanguage {
	fn scope_id(&self) -> Option<&str> {
		self.0.id()
	}

	fn apply<'c>(&self, stats: &mut StatsBuilder<'c>) {
		let language = match &self.0 {
			Selector::Specific(language) => Some(language.clone()),
			_ => stats.get_selection().map(str::to_owned),
		};
		if let Some(lang) = language {
			stats.add_language(lang);
		}
	}
}
