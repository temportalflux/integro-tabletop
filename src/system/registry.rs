use super::System;
use std::{collections::HashMap, sync::Arc};

mod builder;
pub use builder::*;
mod entry;
pub use entry::*;

#[derive(Clone)]
pub struct Registry(Arc<HashMap<&'static str, Entry>>);

impl PartialEq for Registry {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

impl Registry {
	pub fn builder() -> Builder {
		Builder::new()
	}

	pub(in super::registry) fn new(systems: HashMap<&'static str, Entry>) -> Self {
		Self(Arc::new(systems))
	}

	pub fn get_sys<T: System>(&self) -> Option<&Entry> {
		self.get(T::id())
	}

	pub fn get(&self, system_id: &str) -> Option<&Entry> {
		self.0.get(system_id)
	}
}
