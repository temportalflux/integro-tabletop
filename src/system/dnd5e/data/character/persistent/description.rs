use crate::system::dnd5e::data::Size;
use std::collections::HashSet;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Description {
	pub name: String,
	pub pronouns: HashSet<String>,
	pub custom_pronouns: String,
	pub height: u32,
	pub weight: u32,
}

impl Description {
	pub fn size(&self) -> Size {
		match self.height {
			v if v >= 45 => Size::Medium,
			_ => Size::Small,
		}
	}
}
