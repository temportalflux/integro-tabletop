use std::{
	collections::HashMap,
	sync::{Arc, Mutex, MutexGuard},
};

/// A system which can parse a kdl document into its internal structures.
pub trait System {
	fn id() -> &'static str where Self: Sized;
	fn id_owned(&self) -> &'static str;
}

/// Registry of supported tabletop systems, referencable by their system id (e.g. `dnd5e`).
#[derive(Default)]
pub struct SystemRegistry {
	systems: HashMap<&'static str, Arc<Mutex<dyn System + 'static + Send + Sync>>>,
}

impl SystemRegistry {
	/// Add a system to the registry.
	pub fn register<T>(&mut self, system: T)
	where
		T: System + 'static + Send + Sync,
	{
		self.systems
			.insert(system.id_owned(), Arc::new(Mutex::new(system)));
	}

	/// Get a mutable lock on a system by its id.
	pub fn get<'this>(
		&'this self,
		id: &str,
	) -> Option<MutexGuard<'this, dyn System + 'static + Send + Sync>> {
		match self.systems.get(id) {
			None => None,
			Some(arc) => Some(arc.lock().unwrap()),
		}
	}
}
