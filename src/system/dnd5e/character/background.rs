use super::DerivedBuilder;
use crate::system::dnd5e::{
	mutator::{self, BoxedMutator},
	BoxedFeature,
};

#[derive(Clone, PartialEq)]
pub struct Background {
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<BoxedFeature>,
}

impl mutator::Container for Background {
	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		for mutator in &self.mutators {
			stats.apply(mutator);
		}
		for feat in &self.features {
			stats.add_feature(feat);
		}
	}
}
