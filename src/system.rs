use std::{
	collections::HashMap,
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

#[derive(Default)]
pub struct DnD5e {
	node_registry: FactoryRegistry<DnD5e>,
}

impl System for DnD5e {
	fn id(&self) -> &'static str {
		"dnd5e"
	}

	fn insert_document(&mut self, document: kdl::KdlDocument) {
		for node in document.nodes() {
			let node_name = node.name().value();
			let Some(factory) = self.node_registry.get(node_name).cloned() else {
				log::error!("Failed to find factory to deserialize node \"{node_name}\".");
				continue;
			};
			if let Err(err) = factory.deserialize(node, self) {
				log::error!("Failed to deserialize entry: {err:?}");
			}
		}
	}
}

impl DnD5e {
	pub fn with_node_factory(
		mut self,
		node_name: &'static str,
		factory: impl Factory<System = Self> + 'static + Send + Sync,
	) -> Self {
		self.node_registry.register(node_name, factory);
		self
	}
}

#[derive(Default)]
pub struct FactoryRegistry<TSystem> {
	node_factories:
		HashMap<&'static str, Arc<dyn Factory<System = TSystem> + 'static + Send + Sync>>,
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
