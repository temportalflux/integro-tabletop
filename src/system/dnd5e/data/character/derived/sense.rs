use crate::system::dnd5e::data::bounded::{BoundValue, BoundedValue};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Senses(BTreeMap<String, BoundedValue>);
impl Senses {
	pub fn insert(&mut self, kind: String, bound: BoundValue, source: PathBuf) {
		match self.0.get_mut(&kind) {
			Some(value) => {
				value.insert(bound, source);
			}
			None => {
				let mut value = BoundedValue::default();
				value.insert(bound, source);
				self.0.insert(kind, value);
			}
		}
	}
}
impl std::ops::Deref for Senses {
	type Target = BTreeMap<String, BoundedValue>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
