use super::{
	character::StatsBuilder,
	modifier::{self, Modifier},
	Action,
};

#[derive(Default, Clone)]
pub struct Feature {
	pub name: String,
	pub description: String,
	pub action: Option<Action>,
	pub modifiers: Vec<Box<dyn Modifier + 'static>>,
	pub limited_uses: Option<LimitedUses>,
}

impl PartialEq for Feature {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.description == other.description
			&& self.action == other.action
	}
}

impl modifier::Container for Feature {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}

	fn apply_modifiers<'c>(&self, stats: &mut StatsBuilder<'c>) {
		for modifier in &self.modifiers {
			stats.apply(modifier);
		}
	}
}

#[derive(Default, Clone)]
pub struct LimitedUses {}
