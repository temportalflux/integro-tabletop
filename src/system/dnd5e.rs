use self::data::character::Character;
use super::{Block, BlockRegistry, NodeRegistry};
use std::sync::Arc;

pub mod components;
pub mod data;
pub mod evaluator;
pub mod generator;
pub mod mutator;

pub type BoxedCriteria = crate::utility::GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = crate::utility::GenericEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::GenericMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;
pub use crate::system::Block as SystemBlock;

impl Block for crate::utility::GenericGenerator {
	fn to_metadata(self) -> serde_json::Value {
		// TODO: id (SourceId) and kind (<Generator as NodeId>::id) fields
		serde_json::json!(null)
	}
}

pub fn block_registry() -> BlockRegistry {
	let mut registry = BlockRegistry::default();
	registry.register::<data::character::DefaultsBlock>();
	registry.register::<data::character::Persistent>();
	registry.register::<data::Bundle>();
	registry.register::<data::Class>();
	registry.register::<data::Subclass>();
	registry.register::<data::Condition>();
	registry.register::<data::item::Item>();
	registry.register::<data::Spell>();
	registry.register::<crate::utility::GenericGenerator>();
	registry
}

pub fn node_registry() -> NodeRegistry {
	use evaluator::*;
	use generator::*;
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

	// Order matters here! Block generators are first because they can make other generators.
	// This order instructs the priority queue to the order in which generators are processed.
	registry.register_generator::<BlockGenerator>();
	registry.register_generator::<ItemGenerator>();

	registry
}

pub struct DnD5e {
	blocks: BlockRegistry,
	generics: Arc<NodeRegistry>,
}

impl DnD5e {
	pub fn new() -> Self {
		Self {
			blocks: block_registry(),
			generics: node_registry().into(),
		}
	}
}

impl super::System for DnD5e {
	fn id() -> &'static str {
		"dnd5e"
	}

	fn get_id(&self) -> &'static str {
		Self::id()
	}

	fn blocks(&self) -> &BlockRegistry {
		&self.blocks
	}

	fn generics(&self) -> &Arc<NodeRegistry> {
		&self.generics
	}
}
