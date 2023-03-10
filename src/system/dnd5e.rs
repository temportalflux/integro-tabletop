use self::data::character::Character;
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::core::NodeRegistry,
};
use std::{collections::HashMap, sync::Arc};

use super::core::SourceId;

pub mod components;
pub mod content;
pub mod data;

pub type BoxedCriteria = crate::utility::GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = crate::utility::GenericEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::GenericMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

struct ComponentFactory {
	add_from_kdl: Box<
		dyn Fn(&kdl::KdlNode, SourceId, &mut DnD5e) -> anyhow::Result<()>
			+ 'static
			+ Send
			+ Sync,
	>,
}
impl ComponentFactory {
	fn new<T>() -> Self
	where
		T: FromKDL + SystemComponent<System = DnD5e> + 'static + Send + Sync,
	{
		Self {
			add_from_kdl: Box::new(|node, source_id, system| {
				let value = T::from_kdl(node, &mut ValueIdx::default(), &system.node_registry)?;
				T::add_component(value, source_id, system);
				Ok(())
			}),
		}
	}

	fn add_from_kdl(
		&self,
		node: &kdl::KdlNode,
		source_id: SourceId,
		system: &mut DnD5e,
	) -> anyhow::Result<()> {
		(*self.add_from_kdl)(node, source_id, system)?;
		Ok(())
	}
}

pub trait KDLNode {
	fn id() -> &'static str
	where
		Self: Sized;
}

pub trait FromKDL {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self>
	where
		Self: Sized;
}
macro_rules! impl_fromkdl {
	($target:ty, $method:ident, $map:expr) => {
		impl FromKDL for $target {
			fn from_kdl(
				node: &kdl::KdlNode,
				value_idx: &mut ValueIdx,
				_: &NodeRegistry,
			) -> anyhow::Result<Self> {
				Ok(node.$method(value_idx.next()).map($map)?)
			}
		}
	};
}
impl_fromkdl!(bool, get_bool, |v| v);
impl_fromkdl!(u8, get_i64, |v| v as u8);
impl_fromkdl!(i8, get_i64, |v| v as i8);
impl_fromkdl!(u16, get_i64, |v| v as u16);
impl_fromkdl!(i16, get_i64, |v| v as i16);
impl_fromkdl!(u32, get_i64, |v| v as u32);
impl_fromkdl!(i32, get_i64, |v| v as i32);
impl_fromkdl!(u64, get_i64, |v| v as u64);
impl_fromkdl!(i64, get_i64, |v| v);
impl_fromkdl!(u128, get_i64, |v| v as u128);
impl_fromkdl!(i128, get_i64, |v| v as i128);
impl_fromkdl!(usize, get_i64, |v| v as usize);
impl_fromkdl!(isize, get_i64, |v| v as isize);
impl_fromkdl!(f32, get_f64, |v| v as f32);
impl_fromkdl!(f64, get_f64, |v| v);
impl FromKDL for String {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_: &NodeRegistry,
	) -> anyhow::Result<Self> {
		Ok(node.get_str(value_idx.next())?.to_string())
	}
}
impl<T> FromKDL for Option<T>
where
	T: FromKDL,
{
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		// Instead of consuming the next-idx, just peek to see if there is a value there or not.
		match node.get(**value_idx) {
			Some(_) => T::from_kdl(node, value_idx, node_reg).map(|v| Some(v)),
			None => Ok(None),
		}
	}
}

pub trait SystemComponent {
	type System;

	fn add_component(self, source_id: SourceId, system: &mut Self::System)
	where
		Self: Sized;
}

#[derive(Default)]
pub struct DnD5e {
	root_registry: HashMap<&'static str, Arc<ComponentFactory>>,
	node_registry: NodeRegistry,
	pub lineages: HashMap<SourceId, data::Lineage>,
	pub items: HashMap<SourceId, data::item::Item>,
}

impl super::core::System for DnD5e {
	fn id(&self) -> &'static str {
		"dnd5e"
	}

	fn insert_document(
		&mut self,
		mut source_id: SourceId,
		document: kdl::KdlDocument,
	) {
		for (idx, node) in document.nodes().iter().enumerate() {
			source_id.node_idx = idx;
			let node_name = node.name().value();
			if node_name == "system" {
				continue;
			}
			let Some(comp_factory) = self.root_registry.get(node_name).cloned() else {
				log::error!("Failed to find factory to deserialize node \"{node_name}\".");
				continue;
			};
			if let Err(err) = comp_factory.add_from_kdl(node, source_id.clone(), self) {
				log::error!("Failed to deserialize entry: {err:?}");
			}
		}
	}
}

impl DnD5e {
	pub fn new() -> Self {
		let mut system = Self::default();
		system.register_component::<data::Lineage>();
		system.register_component::<data::item::Item>();
		//system.register_mutator::<data::mutator::AddAbilityScore>();
		//system.register_mutator::<data::mutator::AddArmorClassFormula>();
		//system.register_mutator::<data::mutator::AddAction>();
		//system.register_mutator::<data::mutator::BonusDamage>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddDefense>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddLifeExpectancy>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddMaxHeight>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddMaxHitPoints>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddProficiency>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddSavingThrowModifier>();
		system
			.node_registry
			.register_mutator::<data::mutator::AddSkillModifier>();
		system
			.node_registry
			.register_mutator::<data::mutator::Speed>();
		system
			.node_registry
			.register_mutator::<data::mutator::Sense>();
		system
			.node_registry
			.register_mutator::<data::mutator::SetFlag>();
		system
			.node_registry
			.register_evaluator::<data::evaluator::armor::HasArmorEquipped>();
		system
			.node_registry
			.register_evaluator::<data::evaluator::GetAbilityModifier>();
		system
			.node_registry
			.register_evaluator::<data::evaluator::GetLevel>();
		system
			.node_registry
			.register_evaluator::<data::evaluator::IsProficientWith>();
		system
	}

	pub fn register_component<T>(&mut self)
	where
		T: FromKDL + KDLNode + SystemComponent<System = Self> + 'static + Send + Sync,
	{
		assert!(!self.root_registry.contains_key(T::id()));
		self.root_registry
			.insert(T::id(), ComponentFactory::new::<T>().into());
	}
}
