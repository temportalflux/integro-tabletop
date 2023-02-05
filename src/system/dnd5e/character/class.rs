use super::StatsBuilder;
use crate::system::dnd5e::modifier;

#[derive(Clone)]
pub struct Class {
	pub name: String,
}

impl modifier::Container for Class {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}

	fn apply_modifiers<'c>(&self, stats: &mut StatsBuilder<'c>) {}
}
