use super::GetLevel;
use crate::{system::dnd5e::data::character::Character, utility::Evaluator};
use std::{collections::BTreeMap, fmt::Debug};

#[derive(Clone, PartialEq, Debug)]
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

impl<T> crate::utility::TraitEq for ByLevel<T>
where
	T: 'static + PartialEq,
{
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl<T> Evaluator for ByLevel<T>
where
	T: 'static + Clone + Default + Send + Sync + Debug + PartialEq,
{
	type Context = Character;
	type Item = T;

	fn evaluate(&self, state: &Self::Context) -> Self::Item {
		let character_level = GetLevel::from(self.class_name.clone()).evaluate(state);
		for (level, value) in self.map.iter().rev() {
			if *level <= character_level {
				return value.clone();
			}
		}
		T::default()
	}
}
