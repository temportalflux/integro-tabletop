use super::StatsBuilder;
use crate::system::dnd5e::{modifier, roll::Die};

#[derive(Clone)]
pub struct Class {
	pub name: String,
	pub hit_die: Die,
}

impl Class {
	pub fn level_count(&self) -> i32 {
		// TODO
		0
	}
}

impl modifier::Container for Class {
	fn id(&self) -> String {
		use convert_case::Casing;
		self.name.to_case(convert_case::Case::Pascal)
	}

	fn apply_modifiers<'c>(&self, _stats: &mut StatsBuilder<'c>) {}
}
