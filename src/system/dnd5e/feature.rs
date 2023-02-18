use super::{
	character::DerivedBuilder,
	mutator::{self, BoxedMutator},
	Action,
};
use std::rc::Rc;

#[derive(Default, Clone, PartialEq)]
pub struct Feature {
	pub name: String,
	pub description: String,
	pub action: Option<Action>,
	pub modifiers: Vec<BoxedMutator>,
	pub limited_uses: Option<LimitedUses>,
}

impl mutator::Container for Feature {
	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		for modifier in &self.modifiers {
			stats.apply(modifier);
		}
	}
}

#[derive(Clone, PartialEq)]
pub struct BoxedFeature(Rc<Feature>);
impl From<Feature> for BoxedFeature {
	fn from(feature: Feature) -> Self {
		Self(Rc::new(feature))
	}
}
impl BoxedFeature {
	pub fn inner(&self) -> &Feature {
		&*self.0
	}
}

#[derive(Default, Clone, PartialEq)]
pub struct LimitedUses {}
