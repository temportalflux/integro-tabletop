use self::data::character::Character;
use super::core::SourceId;
use crate::{
	kdl_ext::{NodeExt, ValueIdx},
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
	dyn Fn(&kdl::KdlNode, SourceId, &NodeRegistry) -> anyhow::Result<FnInsertComp<S>>
		+ 'static
		+ Send
		+ Sync,
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
			add_from_kdl: Box::new(|node, source_id, node_reg| {
				let value = T::from_kdl(node, &mut ValueIdx::default(), node_reg)?;
				Ok(Box::new(|system| {
					T::add_component(value, source_id, system);
				}))
			}),
		}
	}

	pub fn add_from_kdl(
		&self,
		node: &kdl::KdlNode,
		source_id: SourceId,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<FnInsertComp<S>> {
		(*self.add_from_kdl)(node, source_id, node_reg)
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

pub trait KDLNode {
	fn id() -> &'static str
	where
		Self: Sized;

	fn get_id(&self) -> &'static str;
}

#[macro_export]
macro_rules! impl_kdl_node {
	($target:ty, $id:expr) => {
		impl crate::system::dnd5e::KDLNode for $target {
			fn id() -> &'static str {
				$id
			}

			fn get_id(&self) -> &'static str {
				$id
			}
		}
	};
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
impl_fromkdl!(bool, get_bool_req, |v| v);
impl_fromkdl!(u8, get_i64_req, |v| v as u8);
impl_fromkdl!(i8, get_i64_req, |v| v as i8);
impl_fromkdl!(u16, get_i64_req, |v| v as u16);
impl_fromkdl!(i16, get_i64_req, |v| v as i16);
impl_fromkdl!(u32, get_i64_req, |v| v as u32);
impl_fromkdl!(i32, get_i64_req, |v| v as i32);
impl_fromkdl!(u64, get_i64_req, |v| v as u64);
impl_fromkdl!(i64, get_i64_req, |v| v);
impl_fromkdl!(u128, get_i64_req, |v| v as u128);
impl_fromkdl!(i128, get_i64_req, |v| v as i128);
impl_fromkdl!(usize, get_i64_req, |v| v as usize);
impl_fromkdl!(isize, get_i64_req, |v| v as isize);
impl_fromkdl!(f32, get_f64_req, |v| v as f32);
impl_fromkdl!(f64, get_f64_req, |v| v);
impl FromKDL for String {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		_: &NodeRegistry,
	) -> anyhow::Result<Self> {
		Ok(node.get_str_req(value_idx.next())?.to_string())
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

pub fn component_registry() -> ComponentRegistry<DnD5e> {
	let mut registry = ComponentRegistry::default();
	registry.register::<data::bundle::Race>();
	registry.register::<data::bundle::RaceVariant>();
	registry.register::<data::bundle::Lineage>();
	registry.register::<data::bundle::Upbringing>();
	registry.register::<data::bundle::Background>();
	registry.register::<data::Class>();
	registry.register::<data::Subclass>();
	registry.register::<data::item::Item>();
	registry
}

pub fn node_registry() -> NodeRegistry {
	use data::{evaluator::*, mutator::*};
	let mut registry = NodeRegistry::default();
	registry.register_mutator::<AbilityScoreChange>();
	registry.register_mutator::<AddArmorClassFormula>();
	registry.register_mutator::<AddDefense>();
	registry.register_mutator::<AddLifeExpectancy>();
	registry.register_mutator::<AddMaxHeight>();
	registry.register_mutator::<AddMaxHitPoints>();
	registry.register_mutator::<AddProficiency>();
	registry.register_mutator::<AddModifier>();
	registry.register_mutator::<Speed>();
	registry.register_mutator::<Sense>();
	registry.register_mutator::<SetFlag>();
	registry.register_evaluator::<armor::HasArmorEquipped>();
	registry.register_evaluator::<GetAbilityModifier>();
	registry.register_evaluator::<GetLevel>();
	registry.register_evaluator::<IsProficientWith>();
	registry
}

#[derive(Clone, PartialEq, Default)]
pub struct DnD5e {
	pub races: HashMap<SourceId, data::bundle::Race>,
	pub race_variants: HashMap<SourceId, data::bundle::RaceVariant>,
	pub lineages: HashMap<SourceId, data::bundle::Lineage>,
	pub upbringings: HashMap<SourceId, data::bundle::Upbringing>,
	pub backgrounds: HashMap<SourceId, data::bundle::Background>,
	pub classes: HashMap<SourceId, data::Class>,
	pub subclasses: HashMap<SourceId, data::Subclass>,
	pub items: HashMap<SourceId, data::item::Item>,
}

impl super::core::System for DnD5e {
	fn id(&self) -> &'static str {
		"dnd5e"
	}
}
