use std::path::PathBuf;

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
impl<T> From<(T, Vec<(PathBuf, T)>)> for AttributedValue<T> {
	fn from((value, sources): (T, Vec<(PathBuf, T)>)) -> Self {
		Self { value, sources }
	}
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
