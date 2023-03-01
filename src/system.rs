use std::{
	collections::HashMap,
	default,
	sync::{Arc, Mutex, MutexGuard},
};

pub mod dnd5e;

#[derive(Default)]
pub struct SystemRegistry {
	systems: HashMap<&'static str, Arc<Mutex<dyn System + 'static + Send + Sync>>>,
}
impl SystemRegistry {
	pub fn register<T>(&mut self, system: T)
	where
		T: System + 'static + Send + Sync,
	{
		self.systems
			.insert(system.id(), Arc::new(Mutex::new(system)));
	}

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

pub trait System {
	fn id(&self) -> &'static str;
	fn insert_document(&mut self, _document: kdl::KdlDocument) {}
}

pub struct FactoryRegistry<TSystem> {
	node_factories:
		HashMap<&'static str, Arc<dyn Factory<System = TSystem> + 'static + Send + Sync>>,
}
impl<TSystem> Default for FactoryRegistry<TSystem>
where
	TSystem: 'static + Default + System + Send + Sync,
{
	fn default() -> Self {
		let mut registry = Self {
			node_factories: HashMap::default(),
		};
		registry.register("system", FactoryNull::<TSystem>::default());
		registry
	}
}
impl<TSystem> FactoryRegistry<TSystem>
where
	TSystem: System,
{
	fn register(
		&mut self,
		node_name: &'static str,
		factory: impl Factory<System = TSystem> + 'static + Send + Sync,
	) {
		self.node_factories.insert(node_name, Arc::new(factory));
	}

	fn get(
		&self,
		node_name: &str,
	) -> Option<&Arc<dyn Factory<System = TSystem> + 'static + Send + Sync>> {
		self.node_factories.get(node_name)
	}
}

pub trait Factory {
	type System: System;
	fn deserialize(&self, node: &kdl::KdlNode, system: &mut Self::System) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct FactoryNull<TSystem>(std::marker::PhantomData<TSystem>);
impl<TSystem> Factory for FactoryNull<TSystem>
where
	TSystem: System,
{
	type System = TSystem;
	fn deserialize(&self, node: &kdl::KdlNode, system: &mut Self::System) -> anyhow::Result<()> {
		Ok(())
	}
}
