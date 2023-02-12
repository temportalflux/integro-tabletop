use super::{
	character::DerivedBuilder,
	modifier::{self, BoxedModifier},
	Action,
};

#[derive(Default, Clone, PartialEq)]
pub struct Feature {
	pub name: String,
	pub description: String,
	pub action: Option<Action>,
	pub modifiers: Vec<BoxedModifier>,
	pub limited_uses: Option<LimitedUses>,
}

impl modifier::Container for Feature {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}

	fn apply_modifiers<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		for modifier in &self.modifiers {
			stats.apply(modifier);
		}
	}
}

#[derive(Default, Clone, PartialEq)]
pub struct LimitedUses {}
