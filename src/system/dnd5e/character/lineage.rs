use super::{DerivedBuilder, Feature};
use crate::system::dnd5e::modifier;

#[derive(Default, Clone, PartialEq)]
pub struct Lineage {
	pub name: String,
	pub description: String,
	pub features: Vec<Feature>,
}

impl modifier::Container for Lineage {
	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_modifiers<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		for feat in &self.features {
			stats.apply_from(feat);
		}
	}
}
