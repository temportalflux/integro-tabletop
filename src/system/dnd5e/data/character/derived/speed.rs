use crate::system::{
	dnd5e::data::bounded::{BoundValue, BoundedValue},
	mutator::ReferencePath,
};
use std::collections::BTreeMap;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Speeds(BTreeMap<String, BoundedValue>);
impl Speeds {
	pub fn insert(&mut self, kind: String, bound: BoundValue, source: &ReferencePath) {
		match self.0.get_mut(&kind) {
			Some(value) => {
				value.insert(bound, source.display.clone());
			}
			None => {
				let mut value = BoundedValue::default();
				value.insert(bound, source.display.clone());
				self.0.insert(kind, value);
			}
		}
	}
}
impl std::ops::Deref for Speeds {
	type Target = BTreeMap<String, BoundedValue>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
