use super::DerivedBuilder;
use crate::system::dnd5e::{modifier, BoxedFeature};

#[derive(Clone, PartialEq)]
pub struct Background {
	pub name: String,
	pub description: String,
	pub features: Vec<BoxedFeature>,
}

impl modifier::Container for Background {
	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_modifiers<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		for feat in &self.features {
			stats.add_feature(feat);
		}
	}
}
