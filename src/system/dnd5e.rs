use self::data::character::Character;
use crate::{
	utility::{ArcEvaluator, Evaluator, GenericEvaluator, Mutator},
	GeneralError,
};
use std::{any::Any, collections::HashMap, sync::Arc};

pub mod components;
pub mod content;
pub mod data;

pub type BoxedCriteria = GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = GenericEvaluator<Character, V>;
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

pub trait KDLNode {
	fn id() -> &'static str
	where
		Self: Sized;
}

pub trait FromKDL {
	type System;
	fn from_kdl(node: &kdl::KdlNode, system: &Self::System) -> anyhow::Result<Self>
	where
		Self: Sized;
}
pub trait SystemComponent {
	type System;

	fn add_component(self, system: &mut Self::System)
	where
		Self: Sized;
}

pub struct FromKDLFactory<T, S> {
	fn_from_kdl: Box<dyn Fn(&kdl::KdlNode, &S) -> anyhow::Result<T> + 'static + Send + Sync>,
}
impl<T, S> FromKDLFactory<T, S> {
	fn new<U>() -> Self
	where
		U: FromKDL<System = S> + Into<T> + 'static + Send + Sync,
	{
		Self {
			fn_from_kdl: Box::new(|node, system| Ok(U::from_kdl(node, system)?.into())),
		}
	}

	pub fn from_kdl(&self, node: &kdl::KdlNode, system: &S) -> anyhow::Result<T> {
		(*self.fn_from_kdl)(node, system)
	}
}

type BoxAny = Box<dyn Any + 'static + Send + Sync>;
pub struct EvaluatorFactory {
	fn_from_kdl:
		Box<dyn Fn(&kdl::KdlNode, &DnD5e) -> anyhow::Result<BoxAny> + 'static + Send + Sync>,
}
impl EvaluatorFactory {
	pub fn new<E>() -> Self
	where
		E: Evaluator<Context = Character> + FromKDL<System = DnD5e> + 'static + Send + Sync,
	{
		Self {
			fn_from_kdl: Box::new(|node, system| {
				let arc_eval: ArcEvaluator<E::Context, E::Item> =
					Arc::new(E::from_kdl(node, system)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<T>(
		&self,
		node: &kdl::KdlNode,
		system: &DnD5e,
	) -> anyhow::Result<GenericEvaluator<Character, T>>
	where
		T: 'static,
	{
		let any = (self.fn_from_kdl)(node, system)?;
		let eval = any
			.downcast::<ArcEvaluator<Character, T>>()
			.expect("failed to unpack boxed arc-evaluator");
		Ok(GenericEvaluator::new(*eval))
	}
}

#[derive(Default)]
pub struct DnD5e {
	root_registry: HashMap<&'static str, Arc<ComponentFactory>>,
	mutator_registry: HashMap<&'static str, FromKDLFactory<BoxedMutator, DnD5e>>,
	evaluator_registry: HashMap<&'static str, EvaluatorFactory>,
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
		/*
		system.register_mutator::<data::AddProficiency>();
		system.register_mutator::<data::mutator::AddAbilityScore>();
		system.register_mutator::<data::mutator::AddArmorClassFormula>();
		system.register_mutator::<data::mutator::AddAction>();
		system.register_mutator::<data::mutator::BonusDamage>();
		system.register_mutator::<data::mutator::AddDefense>();
		system.register_mutator::<data::mutator::AddLifeExpectancy>();
		system.register_mutator::<data::mutator::AddMaxHeight>();
		system.register_mutator::<data::mutator::AddMaxHitPoints>();
		system.register_mutator::<data::mutator::AddSavingThrow>();
		system.register_mutator::<data::mutator::AddSkill>();
		system.register_mutator::<data::mutator::AddSkillModifier>();
		system.register_mutator::<data::mutator::AddMaxSpeed>();
		system.register_mutator::<data::mutator::AddMaxSense>();
		*/
		system.register_evaluator::<data::evaluator::armor::HasArmorEquipped>();
		system
	}

	pub fn register_component<T>(&mut self)
	where
		T: FromKDL<System = Self>
			+ KDLNode
			+ SystemComponent<System = Self>
			+ 'static
			+ Send
			+ Sync,
	{
		assert!(!self.root_registry.contains_key(T::id()));
		self.root_registry
			.insert(T::id(), ComponentFactory::new::<T>().into());
	}

	pub fn register_mutator<T>(&mut self)
	where
		T: Mutator<Target = data::character::Character>
			+ KDLNode
			+ FromKDL<System = DnD5e>
			+ 'static
			+ Send
			+ Sync,
	{
		assert!(!self.mutator_registry.contains_key(T::id()));
		self.mutator_registry
			.insert(T::id(), FromKDLFactory::new::<T>());
	}

	pub fn register_evaluator<E>(&mut self)
	where
		E: Evaluator<Context = Character>
			+ KDLNode
			+ FromKDL<System = DnD5e>
			+ 'static
			+ Send
			+ Sync,
	{
		assert!(!self.evaluator_registry.contains_key(E::id()));
		self.evaluator_registry
			.insert(E::id(), EvaluatorFactory::new::<E>());
	}

	pub fn get_mutator_factory(
		&self,
		id: &str,
	) -> anyhow::Result<&FromKDLFactory<BoxedMutator, DnD5e>> {
		self.mutator_registry
			.get(id)
			.ok_or(GeneralError(format!("No mutator registration found for {id:?}.")).into())
	}

	pub fn get_evaluator_factory(&self, id: &str) -> anyhow::Result<&EvaluatorFactory> {
		self.evaluator_registry
			.get(id)
			.ok_or(GeneralError(format!("No evaluator registration found for {id:?}.")).into())
	}

	pub fn add_lineage(&mut self, lineage: data::Lineage) {
		self.lineages.push(lineage);
	}
}
