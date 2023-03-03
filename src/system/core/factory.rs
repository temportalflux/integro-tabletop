use super::System;
use std::{collections::HashMap, sync::Arc};

/// A constructor which can parse a kdl node, and insert the deserialized structure into a [`System`].
pub trait Factory {
	type System: System;
	fn deserialize(&self, node: &kdl::KdlNode, system: &mut Self::System) -> anyhow::Result<()>;
}

/// A registry of node [`Factory`] parsers that can be used to transform a node into data for a system.
/// Leverages the name of a node as the registry key, example below:
/// ```kdl
/// system "my-system"
/// character name="Sid the Squid" {
/// 	...
/// }
/// ```
/// And to parse the character node into a structure:
/// ```ignore
/// let kdl_node = ...;
/// let mut factory = FactoryRegistry::<MySystem>::default();
/// factory.register("character", FactoryNull::<MySystem>::default());
/// factory.get("character").unwrap().deserialize(kdl_node, my_system);
/// ```
pub struct FactoryRegistry<TSystem>(
	HashMap<&'static str, Arc<dyn Factory<System = TSystem> + 'static + Send + Sync>>,
);

impl<TSystem> Default for FactoryRegistry<TSystem>
where
	TSystem: 'static + Default + System + Send + Sync,
{
	fn default() -> Self {
		let mut registry = Self(HashMap::default());
		registry.register("system", FactoryNull::<TSystem>::ignored());
		registry
	}
}

impl<TSystem> FactoryRegistry<TSystem>
where
	TSystem: System,
{
	/// Registers a node-parsing factory for the system.
	/// The key should be the name of the node as it will be used in kdl text.
	pub fn register(
		&mut self,
		node_name: &'static str,
		factory: impl Factory<System = TSystem> + 'static + Send + Sync,
	) {
		self.0.insert(node_name, Arc::new(factory));
	}

	/// Returns the factory for a given registered node-name.
	pub fn get(
		&self,
		node_name: &str,
	) -> Option<&Arc<dyn Factory<System = TSystem> + 'static + Send + Sync>> {
		self.0.get(node_name)
	}
}

/// A stub factory which can be registered with any name in a [`FactoryRegistry`],
/// but only logs a warning that the factory is not implemented when a node is visited.
pub struct FactoryNull<TSystem> {
	log_warning: bool,
	marker: std::marker::PhantomData<TSystem>,
}

impl<TSystem> Default for FactoryNull<TSystem> {
	fn default() -> Self {
		Self {
			log_warning: true,
			marker: Default::default(),
		}
	}
}

impl<TSystem> FactoryNull<TSystem> {
	/// Creates a factory which does not log a warning when a node is visited.
	pub fn ignored() -> Self {
		Self {
			log_warning: false,
			..Default::default()
		}
	}
}

impl<TSystem> Factory for FactoryNull<TSystem>
where
	TSystem: System,
{
	type System = TSystem;
	fn deserialize(&self, node: &kdl::KdlNode, _system: &mut Self::System) -> anyhow::Result<()> {
		if self.log_warning {
			log::warn!("Node named {:?} is registered with the null-factory, and is therefore not actually deserialized.", node.name().value());
		}
		Ok(())
	}
}
