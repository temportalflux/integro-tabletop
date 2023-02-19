use super::Evaluator;
use std::collections::BTreeMap;

pub struct ByClassLevel<T>(BTreeMap<u32, T>);

impl<T, const N: usize> From<[(u32, T); N]> for ByClassLevel<T> {
	fn from(value: [(u32, T); N]) -> Self {
		Self(BTreeMap::from(value))
	}
}

impl<T> Evaluator for ByClassLevel<T> {
	type Item = T;
}
