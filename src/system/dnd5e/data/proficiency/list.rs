use super::Level;
use enum_map::EnumMap;
use std::path::PathBuf;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct List(EnumMap<Level, Vec<PathBuf>>);

impl std::ops::Index<Level> for List {
	type Output = Vec<PathBuf>;
	fn index(&self, index: Level) -> &Self::Output {
		&self.0[index]
	}
}

impl List {
	pub fn push(&mut self, level: Level, source: PathBuf) {
		self.0[level].push(source);
	}

	pub fn value(&self) -> Level {
		let iter = self.0.iter();
		let iter = iter.filter(|(_level, sources)| !sources.is_empty());
		let iter = iter.map(|(level, _)| level);
		iter.max().unwrap_or(Level::None)
	}

	pub fn iter(&self) -> impl Iterator<Item = (Level, &PathBuf)> {
		let iter = self.0.iter();
		let iter = iter.map(|(level, sources)| sources.iter().map(move |source| (level, source)));
		iter.flatten()
	}
}

impl FromIterator<(Level, Vec<PathBuf>)> for List {
	fn from_iter<T: IntoIterator<Item = (Level, Vec<PathBuf>)>>(iter: T) -> Self {
		Self(EnumMap::from_iter(iter))
	}
}
