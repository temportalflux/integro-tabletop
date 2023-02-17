use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
};

#[derive(Clone, PartialEq)]
pub struct PathMap<T> {
	values: Vec<T>,
	children: BTreeMap<String, PathMap<T>>,
}

impl<T> Default for PathMap<T> {
	fn default() -> Self {
		Self {
			values: Vec::new(),
			children: BTreeMap::new(),
		}
	}
}

impl<T> PathMap<T> {
	pub fn len(&self) -> usize {
		self.children
			.iter()
			.fold(self.values.len(), |count, (_, child)| count + child.len())
	}

	fn get_mut_or_insert_at(&mut self, path: &Path) -> &mut PathMap<T> {
		let mut layer = self;
		for component in path.components() {
			let key = component.as_os_str().to_str().unwrap();
			if !layer.children.contains_key(key) {
				layer.children.insert(key.to_owned(), PathMap::default());
			}
			layer = layer.children.get_mut(key).unwrap();
		}
		layer
	}

	pub fn insert(&mut self, key: impl AsRef<Path>, value: T) {
		self.get_mut_or_insert_at(key.as_ref()).values.push(value);
	}

	pub fn as_vec(&self) -> Vec<(PathBuf, &T)> {
		let mut all_entries = Vec::with_capacity(self.len());
		self.push_as_vec_entries_to(&PathBuf::new(), &mut all_entries);
		all_entries
	}

	fn push_as_vec_entries_to<'c>(&'c self, path: &PathBuf, target: &mut Vec<(PathBuf, &'c T)>) {
		for value in &self.values {
			target.push((path.clone(), value));
		}
		for (key, child) in &self.children {
			child.push_as_vec_entries_to(&path.join(key), target);
		}
	}

	pub fn iter_values(&self) -> impl Iterator<Item = &T> {
		self.values.iter()
	}

	pub fn iter_children(&self) -> impl Iterator<Item = (&String, &PathMap<T>)> {
		self.children.iter()
	}
}
