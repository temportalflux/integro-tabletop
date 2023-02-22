use crate::system::dnd5e::character::Character;

use super::Evaluator;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq)]
pub struct ByLevel<T> {
	pub class_name: Option<String>,
	pub map: BTreeMap<usize, T>,
}

impl<T, const N: usize> From<[(usize, T); N]> for ByLevel<T> {
	fn from(value: [(usize, T); N]) -> Self {
		Self {
			class_name: None,
			map: BTreeMap::from(value),
		}
	}
}

impl<T> Evaluator for ByLevel<T>
where
	T: Clone + Default,
{
	type Item = T;

	fn evaluate(&self, state: &Character) -> Self::Item {
		let class_name = self.class_name.as_ref().map(String::as_str);
		let character_level = state.level(class_name);
		for (level, value) in self.map.iter().rev() {
			if *level <= character_level {
				return value.clone();
			}
		}
		T::default()
	}
}
