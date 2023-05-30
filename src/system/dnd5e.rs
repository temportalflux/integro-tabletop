use self::data::character::Character;
use super::core::SourceId;
use crate::{
	kdl_ext::{FromKDL, KDLNode, NodeContext},
	system::core::NodeRegistry,
};
use std::{collections::HashMap, sync::Arc};

pub mod components;
pub mod data;
pub mod evaluator;
pub mod mutator;

pub type BoxedCriteria = crate::utility::GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = crate::utility::GenericEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::GenericMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

type FnMetadataFromKdl = Box<
	dyn Fn(&kdl::KdlNode, &NodeContext) -> anyhow::Result<serde_json::Value>
		+ 'static
		+ Send
		+ Sync,
>;
pub struct ComponentFactory {
	metadata_from_kdl: FnMetadataFromKdl,
}
impl ComponentFactory {
	fn new<T>() -> Self
	where
		T: FromKDL + SystemComponent + 'static + Send + Sync,
	{
		Self {
			metadata_from_kdl: Box::new(|node, context| {
				let value = T::from_kdl(node, &mut context.next_node())?;
				Ok(T::to_metadata(value))
			}),
		}
	}

	pub fn metadata_from_kdl(
		&self,
		node: &kdl::KdlNode,
		ctx: &NodeContext,
	) -> anyhow::Result<serde_json::Value> {
		(*self.metadata_from_kdl)(node, ctx)
	}
}
#[derive(Default)]
pub struct ComponentRegistry(HashMap<&'static str, Arc<ComponentFactory>>);
impl ComponentRegistry {
	pub fn register<T>(&mut self)
	where
		T: FromKDL + KDLNode + SystemComponent + 'static + Send + Sync,
	{
		assert!(!self.0.contains_key(T::id()));
		self.0.insert(T::id(), ComponentFactory::new::<T>().into());
	}

	pub fn get_factory(&self, id: &str) -> Option<&Arc<ComponentFactory>> {
		self.0.get(id)
	}
}

pub trait SystemComponent {
	type System;

	fn to_metadata(self) -> serde_json::Value
	where
		Self: Sized;

	fn add_component(self, source_id: SourceId, system: &mut Self::System)
	where
		Self: Sized;
}

pub fn component_registry() -> ComponentRegistry {
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
	registry.register::<data::Spell>();
	registry.register::<data::Feat>();
	registry
}

pub fn node_registry() -> NodeRegistry {
	use evaluator::*;
	use mutator::*;
	let mut registry = NodeRegistry::default();

	registry.register_mutator::<AbilityScoreChange>();
	registry.register_mutator::<AddArmorClassFormula>();
	registry.register_mutator::<AddDefense>();
	registry.register_mutator::<AddLifeExpectancy>();
	registry.register_mutator::<AddToActionBudget>();
	registry.register_mutator::<AddSize>();
	registry.register_mutator::<SuggestedPersonality>();
	registry.register_mutator::<AddMaxHitPoints>();
	registry.register_mutator::<AddModifier>();
	registry.register_mutator::<AddProficiency>();
	registry.register_mutator::<Sense>();
	registry.register_mutator::<Speed>();
	registry.register_mutator::<SetFlag>();
	registry.register_mutator::<Spellcasting>();
	registry.register_mutator::<GrantByLevel>();
	registry.register_mutator::<PickN>();
	registry.register_mutator::<AddFeature>();

	registry.register_evaluator::<GetAbilityModifier>();
	registry.register_evaluator::<GetProficiencyBonus>();
	registry.register_evaluator::<GetHitPoints>();
	registry.register_evaluator::<GetLevel>();
	registry.register_evaluator::<HasArmorEquipped>();
	registry.register_evaluator::<HasCondition>();
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
	pub spells: HashMap<SourceId, data::Spell>,
}

impl super::core::System for DnD5e {
	fn id() -> &'static str {
		"dnd5e"
	}

	fn id_owned(&self) -> &'static str {
		Self::id()
	}
}
