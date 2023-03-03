use self::data::character::{Character, Persistent};
use crate::{
	utility::{Mutator, RcEvaluator},
	GeneralError,
};
use std::{collections::HashMap, sync::Arc};

pub mod components;
pub mod content;
pub mod data;

pub type BoxedCriteria = RcEvaluator<Persistent, Result<(), String>>;
pub type BoxedEvaluator<V> = RcEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::ArcMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

struct ComponentFactory {
	add_from_kdl:
		Box<dyn Fn(&kdl::KdlNode, &mut DnD5e) -> anyhow::Result<()> + 'static + Send + Sync>,
}
impl ComponentFactory {
	fn new<T>() -> Self
	where
		T: FromKDL<System = DnD5e> + SystemComponent<System = DnD5e> + 'static + Send + Sync,
	{
		Self {
			add_from_kdl: Box::new(|node, system| {
				let value = T::from_kdl(node, &system)?;
				T::add_component(value, system);
				Ok(())
			}),
		}
	}

	fn add_from_kdl(&self, node: &kdl::KdlNode, system: &mut DnD5e) -> anyhow::Result<()> {
		(*self.add_from_kdl)(node, system)?;
		Ok(())
	}
}

pub trait FromKDL {
	type System;
	fn from_kdl(node: &kdl::KdlNode, system: &Self::System) -> anyhow::Result<Self>
	where
		Self: Sized;
}
pub trait SystemComponent {
	type System;

	fn node_name() -> &'static str;

	fn add_component(self, system: &mut Self::System)
	where
		Self: Sized;
}

pub struct MutatorFactory {
	fn_from_kdl:
		Box<dyn Fn(&kdl::KdlNode, &DnD5e) -> anyhow::Result<BoxedMutator> + 'static + Send + Sync>,
}
impl MutatorFactory {
	fn new<T>() -> Self
	where
		T: Mutator<Target = data::character::Character>
			+ FromKDL<System = DnD5e>
			+ 'static
			+ Send
			+ Sync,
	{
		Self {
			fn_from_kdl: Box::new(|node, system| Ok(T::from_kdl(node, system)?.into())),
		}
	}

	pub fn from_kdl(&self, node: &kdl::KdlNode, system: &DnD5e) -> anyhow::Result<BoxedMutator> {
		(*self.fn_from_kdl)(node, system)
	}
}

pub struct FeatureFactory;

#[derive(Default)]
pub struct DnD5e {
	root_registry: HashMap<&'static str, Arc<ComponentFactory>>,
	mutator_registry: HashMap<&'static str, MutatorFactory>,
	feature_registry: HashMap<&'static str, FeatureFactory>,
	lineages: Vec<data::Lineage>,
}

impl super::core::System for DnD5e {
	fn id(&self) -> &'static str {
		"dnd5e"
	}

	fn insert_document(&mut self, document: kdl::KdlDocument) {
		for node in document.nodes() {
			let node_name = node.name().value();
			if node_name == "system" {
				continue;
			}
			let Some(comp_factory) = self.root_registry.get(node_name).cloned() else {
				log::error!("Failed to find factory to deserialize node \"{node_name}\".");
				continue;
			};
			if let Err(err) = comp_factory.add_from_kdl(node, self) {
				log::error!("Failed to deserialize entry: {err:?}");
			}
		}
	}
}

impl DnD5e {
	pub fn new() -> Self {
		let mut system = Self::default();
		system.register_component::<data::Lineage>();
		system
	}

	pub fn register_component<T>(&mut self)
	where
		T: FromKDL<System = Self> + SystemComponent<System = Self> + 'static + Send + Sync,
	{
		assert!(!self.root_registry.contains_key(T::node_name()));
		self.root_registry
			.insert(T::node_name(), ComponentFactory::new::<T>().into());
	}

	pub fn register_mutator<T>(&mut self)
	where
		T: Mutator<Target = data::character::Character>
			+ FromKDL<System = DnD5e>
			+ 'static
			+ Send
			+ Sync,
	{
		assert!(!self.mutator_registry.contains_key(T::node_name()));
		self.mutator_registry
			.insert(T::node_name(), MutatorFactory::new::<T>());
	}

	pub fn get_mutator_factory(&self, id: &str) -> anyhow::Result<&MutatorFactory> {
		self.mutator_registry
			.get(id)
			.ok_or(GeneralError(format!("No mutator registration found for {id:?}.")).into())
	}

	pub fn add_lineage(&mut self, lineage: data::Lineage) {
		self.lineages.push(lineage);
	}
}
