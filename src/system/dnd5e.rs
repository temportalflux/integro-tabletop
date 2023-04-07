use self::data::character::Character;
use super::core::SourceId;
use crate::{
	kdl_ext::{FromKDL, KDLNode, NodeContext},
	system::core::NodeRegistry,
};
use std::{collections::HashMap, sync::Arc};

pub mod components;
pub mod data;

pub type BoxedCriteria = crate::utility::GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = crate::utility::GenericEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::GenericMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

type FnCompFromKdl<S> = Box<
	dyn Fn(&kdl::KdlNode, &NodeContext) -> anyhow::Result<FnInsertComp<S>> + 'static + Send + Sync,
>;
type FnInsertComp<S> = Box<dyn FnOnce(&mut S) + 'static + Send + Sync>;
pub struct ComponentFactory<S> {
	add_from_kdl: FnCompFromKdl<S>,
}
impl<S> ComponentFactory<S> {
	fn new<T>() -> Self
	where
		T: FromKDL + SystemComponent<System = S> + 'static + Send + Sync,
	{
		Self {
			add_from_kdl: Box::new(|node, context| {
				let source_id = context.id().clone();
				let value = T::from_kdl(node, &mut context.next_node())?;
				Ok(Box::new(|system| {
					T::add_component(value, source_id, system);
				}))
			}),
		}
	}

	pub fn add_from_kdl(
		&self,
		node: &kdl::KdlNode,
		ctx: &NodeContext,
	) -> anyhow::Result<FnInsertComp<S>> {
		(*self.add_from_kdl)(node, ctx)
	}
}
#[derive(Default)]
pub struct ComponentRegistry<S>(HashMap<&'static str, Arc<ComponentFactory<S>>>);
impl<S> ComponentRegistry<S> {
	pub fn register<T>(&mut self)
	where
		T: FromKDL + KDLNode + SystemComponent<System = S> + 'static + Send + Sync,
	{
		assert!(!self.0.contains_key(T::id()));
		self.0.insert(T::id(), ComponentFactory::new::<T>().into());
	}

	pub fn get_factory(&self, id: &str) -> Option<&Arc<ComponentFactory<S>>> {
		self.0.get(id)
	}
}

pub trait SystemComponent {
	type System;

	fn add_component(self, source_id: SourceId, system: &mut Self::System)
	where
		Self: Sized;
}

pub fn component_registry() -> ComponentRegistry<DnD5e> {
	let mut registry = ComponentRegistry::default();
	registry.register::<data::character::DefaultsBlock>();
	registry.register::<data::bundle::Race>();
	registry.register::<data::bundle::RaceVariant>();
	registry.register::<data::bundle::Lineage>();
	registry.register::<data::bundle::Upbringing>();
	registry.register::<data::bundle::Background>();
	registry.register::<data::Class>();
	registry.register::<data::Subclass>();
	registry.register::<data::Condition>();
	registry.register::<data::item::Item>();
	registry
}

pub fn node_registry() -> NodeRegistry {
	use data::{
		evaluator::{armor::*, *},
		mutator::*,
	};
	let mut registry = NodeRegistry::default();

	registry.register_mutator::<AbilityScoreChange>();
	registry.register_mutator::<AddAction>();
	registry.register_mutator::<AddArmorClassFormula>();
	registry.register_mutator::<AddDefense>();
	registry.register_mutator::<AddLifeExpectancy>();
	registry.register_mutator::<AddToActionBudget>();
	registry.register_mutator::<AddMaxHeight>();
	registry.register_mutator::<AddMaxHitPoints>();
	registry.register_mutator::<AddModifier>();
	registry.register_mutator::<AddProficiency>();
	registry.register_mutator::<Sense>();
	registry.register_mutator::<Speed>();
	registry.register_mutator::<SetFlag>();

	registry.register_evaluator::<GetAbilityModifier>();
	registry.register_evaluator::<GetHitPoints>();
	registry.register_evaluator::<GetLevel>();
	registry.register_evaluator::<HasArmorEquipped>();
	registry.register_evaluator::<IsProficientWith>();
	registry.register_evaluator::<Math>();

	registry
}

#[derive(Clone, PartialEq, Default)]
pub struct DnD5e {
	pub default_blocks: HashMap<SourceId, data::character::DefaultsBlock>,
	pub races: HashMap<SourceId, data::bundle::Race>,
	pub race_variants: HashMap<SourceId, data::bundle::RaceVariant>,
	pub lineages: HashMap<SourceId, data::bundle::Lineage>,
	pub upbringings: HashMap<SourceId, data::bundle::Upbringing>,
	pub backgrounds: HashMap<SourceId, data::bundle::Background>,
	pub classes: HashMap<SourceId, data::Class>,
	pub subclasses: HashMap<SourceId, data::Subclass>,
	pub conditions: HashMap<SourceId, data::Condition>,
	pub items: HashMap<SourceId, data::item::Item>,
}

impl super::core::System for DnD5e {
	fn id(&self) -> &'static str {
		"dnd5e"
	}
}
