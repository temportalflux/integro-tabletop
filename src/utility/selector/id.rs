use derivative::Derivative;
use std::{
	borrow::Cow,
	path::{Path, PathBuf},
	sync::{Arc, RwLock},
};

#[derive(Clone, Default, Derivative)]
#[derivative(PartialEq)]
pub struct IdPath {
	id: Option<String>,
	is_absolute: bool,
	#[derivative(PartialEq = "ignore")]
	absolute_path: Arc<RwLock<PathBuf>>,
}
impl<T: Into<String>> From<Option<T>> for IdPath {
	fn from(value: Option<T>) -> Self {
		let mut id = value.map(|t| t.into());
		let is_absolute = match &mut id {
			None => false,
			Some(key) => match key.strip_prefix("/") {
				None => false,
				Some(stripped) => {
					*key = stripped.to_owned();
					true
				}
			},
		};
		Self {
			id,
			is_absolute,
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
	pub fn get_id(&self) -> Option<Cow<'_, String>> {
		match (self.is_absolute, &self.id) {
			(_, None) => None,
			(true, Some(key)) => Some(Cow::Owned(format!("/{key}"))),
			(false, Some(key)) => Some(Cow::Borrowed(key)),
		}
	}

	pub fn set_path(&self, path: &Path) {
		let mut path = match self.is_absolute {
			false => path.to_owned(),
			true => PathBuf::new(),
		};
		if let Some(id) = &self.id {
			path.push(id);
		}
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
