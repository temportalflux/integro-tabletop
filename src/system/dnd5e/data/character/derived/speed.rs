use crate::system::dnd5e::data::character::AttributedValue;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Speeds(BTreeMap<String, AttributedValue<i32>>);
impl Speeds {
	pub fn push_min(&mut self, kind: String, max_bound_in_feet: i32, source: PathBuf) {
		match self.0.get_mut(&kind) {
			Some(value) => {
				value.push(max_bound_in_feet, source);
			}
			None => {
				let mut value = AttributedValue::default();
				value.push(max_bound_in_feet, source);
				self.0.insert(kind, value);
			}
		}
	}
}
impl std::ops::Deref for Speeds {
	type Target = BTreeMap<String, AttributedValue<i32>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
