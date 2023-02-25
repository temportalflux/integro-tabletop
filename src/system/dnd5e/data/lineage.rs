use crate::{
	system::dnd5e::{
		data::{character::Character, BoxedFeature},
		BoxedMutator,
	},
	utility::MutatorGroup,
};

#[derive(Default, Clone, PartialEq)]
pub struct Lineage {
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<BoxedFeature>,
}

impl MutatorGroup for Lineage {
	type Target = Character;

	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut Self::Target) {
		for mutator in &self.mutators {
			stats.apply(mutator);
		}
		for feat in &self.features {
			stats.add_feature(feat);
		}
	}
}
