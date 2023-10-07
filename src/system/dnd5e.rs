use self::data::character::Character;
use crate::kdl_ext::NodeContext;
use crate::system::core::NodeRegistry;
use kdlize::{AsKdl, FromKdl, NodeId};
use std::{collections::HashMap, sync::Arc};

pub mod components;
pub mod data;
pub mod evaluator;
pub mod mutator;

pub type BoxedCriteria = crate::utility::GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = crate::utility::GenericEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::GenericMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

type FnMetadataFromKdl =
	Box<dyn Fn(crate::kdl_ext::NodeReader<'_>) -> anyhow::Result<serde_json::Value> + 'static + Send + Sync>;
type FnReserializeKdl =
	Box<dyn Fn(crate::kdl_ext::NodeReader<'_>) -> anyhow::Result<kdl::KdlNode> + 'static + Send + Sync>;
pub struct ComponentFactory {
	metadata_from_kdl: FnMetadataFromKdl,
	reserialize_kdl: FnReserializeKdl,
}
impl ComponentFactory {
	fn new<T>() -> Self
	where
		T: FromKdl<NodeContext> + AsKdl + SystemComponent + 'static + Send + Sync,
		anyhow::Error: From<T::Error>,
	{
		Self {
			metadata_from_kdl: Box::new(|mut node| {
				let value = T::from_kdl(&mut node)?;
				Ok(T::to_metadata(value))
			}),
			reserialize_kdl: Box::new(|mut node| {
				let value = T::from_kdl(&mut node)?;
				Ok(value.as_kdl().build(node.name().value()))
			}),
		}
	}

	pub fn metadata_from_kdl<'doc>(&self, node: crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<serde_json::Value> {
		(*self.metadata_from_kdl)(node)
	}

	pub fn reserialize_kdl<'doc>(&self, node: crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<kdl::KdlNode> {
		(*self.reserialize_kdl)(node)
	}
}
#[derive(Default)]
pub struct ComponentRegistry(HashMap<&'static str, Arc<ComponentFactory>>);
impl ComponentRegistry {
	pub fn register<T>(&mut self)
	where
		T: FromKdl<NodeContext> + NodeId + AsKdl + SystemComponent + 'static + Send + Sync,
		anyhow::Error: From<T::Error>,
	{
		assert!(!self.0.contains_key(T::id()));
		self.0.insert(T::id(), ComponentFactory::new::<T>().into());
	}

	pub fn get_factory(&self, id: &str) -> Option<&Arc<ComponentFactory>> {
		self.0.get(id)
	}
}

pub trait SystemComponent {
	fn to_metadata(self) -> serde_json::Value
	where
		Self: Sized;
}

pub fn component_registry() -> ComponentRegistry {
	let mut registry = ComponentRegistry::default();
	registry.register::<data::character::DefaultsBlock>();
	registry.register::<data::character::Persistent>();
	registry.register::<data::Bundle>();
	registry.register::<data::Class>();
	registry.register::<data::Subclass>();
	registry.register::<data::Condition>();
	registry.register::<data::item::Item>();
	registry.register::<data::Spell>();
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
	registry.register_mutator::<AddStartingEquipment>();
	registry.register_mutator::<AddBundle>();
	registry.register_mutator::<Bonus>();
	registry.register_mutator::<ApplyIf>();

	registry.register_evaluator::<GetAbilityModifier>();
	registry.register_evaluator::<GetProficiencyBonus>();
	registry.register_evaluator::<GetHitPoints>();
	registry.register_evaluator::<GetLevelInt>();
	registry.register_evaluator::<GetLevelStr>();
	registry.register_evaluator::<HasArmorEquipped>();
	registry.register_evaluator::<HasAttack>();
	registry.register_evaluator::<HasCondition>();
	registry.register_evaluator::<IsProficientWith>();
	registry.register_evaluator::<Math>();

	registry
}

#[derive(Clone, PartialEq, Default)]
pub struct DnD5e;

impl super::core::System for DnD5e {
	fn id() -> &'static str {
		"dnd5e"
	}

	fn id_owned(&self) -> &'static str {
		Self::id()
	}
}
