use derivative::Derivative;
use std::{
	path::{Path, PathBuf},
	sync::{Arc, RwLock},
};

#[derive(Clone, Default, Derivative)]
#[derivative(PartialEq)]
pub struct IdPath {
	id: Option<String>,
	#[derivative(PartialEq = "ignore")]
	absolute_path: Arc<RwLock<PathBuf>>,
}
impl<T: Into<String>> From<Option<T>> for IdPath {
	fn from(value: Option<T>) -> Self {
		Self {
			id: value.map(|t| t.into()),
			absolute_path: Arc::new(RwLock::new(PathBuf::new())),
		}
	}
}
impl From<&str> for IdPath {
	fn from(value: &str) -> Self {
		Self::from(Some(value))
	}
}
impl IdPath {
	pub fn get_id(&self) -> Option<&String> {
		self.id.as_ref()
	}

	pub fn set_path(&self, path: &Path) {
		let path = match &self.id {
			Some(id) => path.join(id),
			None => path.to_owned(),
		};
		let path = PathBuf::from(path.to_str().unwrap().replace("\\", "/"));
		*self.absolute_path.write().unwrap() = path;
	}

	pub fn as_path(&self) -> Option<PathBuf> {
		let path = self.absolute_path.read().unwrap().clone();
		if path.to_str() == Some("") {
			return None;
		}
		Some(path)
	}
}
impl std::fmt::Debug for IdPath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"IdPath(id={:?}, path={:?})",
			self.id,
			*self.absolute_path.read().unwrap()
		)
	}
}
