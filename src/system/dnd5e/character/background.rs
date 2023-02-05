use super::StatsBuilder;
use crate::system::dnd5e::{modifier, Feature};

#[derive(Clone)]
pub struct Background {
	pub name: String,
	pub description: String,
	pub features: Vec<Feature>,
}

impl modifier::Container for Background {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}

	fn apply_modifiers<'c>(&self, stats: &mut StatsBuilder<'c>) {
		for feat in &self.features {
			stats.apply_from(feat);
		}
	}
}
