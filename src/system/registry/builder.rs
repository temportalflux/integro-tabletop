use crate::system::{
	registry::{Entry, Registry},
	System,
};
use std::collections::HashMap;

pub struct Builder(HashMap<&'static str, Entry>);

impl Builder {
	pub(super) fn new() -> Self {
		Self(HashMap::default())
	}

	pub fn insert<T>(&mut self, system: T)
	where
		T: System + 'static + Send + Sync,
	{
		self.0.insert(system.get_id(), Entry::new(system));
	}

	pub fn build(self) -> Registry {
		Registry::new(self.0)
	}
}
