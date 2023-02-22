use super::mutator;
use std::path::PathBuf;

mod armor_class;
pub use armor_class::*;
mod background;
pub use background::*;
mod class;
pub use class::*;
mod description;
pub use description::*;
mod lineage;
pub use lineage::*;
mod proficiencies;
pub use proficiencies::*;
mod upbringing;
pub use upbringing::*;

mod persistent;
pub use persistent::*;
mod derived;
pub use derived::*;
mod character;
pub use character::*;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct AttributedValue<T> {
	value: T,
	sources: Vec<(PathBuf, T)>,
}
impl<T> AttributedValue<T>
where
	T: Clone,
{
	pub fn set(&mut self, value: T, source: PathBuf) {
		self.value = value.clone();
		self.sources.push((source, value));
	}

	pub fn push(&mut self, value: T, source: PathBuf)
	where
		T: PartialOrd,
	{
		if self.value < value {
			self.value = value.clone();
		}
		self.sources.push((source, value));
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn sources(&self) -> &Vec<(PathBuf, T)> {
		&self.sources
	}
}

#[derive(Clone, PartialEq)]
pub struct Culture {
	pub lineages: [Lineage; 2],
	pub upbringing: Upbringing,
}
