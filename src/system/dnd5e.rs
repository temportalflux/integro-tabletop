use self::data::character::{Character, Persistent};
use crate::utility::RcEvaluator;

pub mod components;
pub mod content;
pub mod data;

pub type BoxedCriteria = RcEvaluator<Persistent, Result<(), String>>;
pub type BoxedEvaluator<V> = RcEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::RcMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

pub struct DnD5e {
	node_registry: super::FactoryRegistry<DnD5e>,
}

impl Default for DnD5e {
	fn default() -> Self {
		let mut system = Self {
			node_registry: Default::default(),
		};

		system
	}
}

impl super::System for DnD5e {
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
		factory: impl super::Factory<System = Self> + 'static + Send + Sync,
	) -> Self {
		self.node_registry.register(node_name, factory);
		self
	}
}
