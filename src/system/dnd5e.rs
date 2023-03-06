use self::data::character::Character;
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	utility::{ArcEvaluator, Evaluator, GenericEvaluator, Mutator},
	GeneralError,
};
use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::Arc,
};

pub mod components;
pub mod content;
pub mod data;

pub type BoxedCriteria = GenericEvaluator<Character, Result<(), String>>;
pub type BoxedEvaluator<V> = GenericEvaluator<Character, V>;
pub type BoxedMutator = crate::utility::ArcMutator<Character>;
pub type Value<T> = crate::utility::Value<Character, T>;

struct ComponentFactory {
	add_from_kdl: Box<
		dyn Fn(&kdl::KdlNode, &mut ValueIdx, &mut DnD5e) -> anyhow::Result<()>
			+ 'static
			+ Send
			+ Sync,
	>,
}
impl ComponentFactory {
	fn new<T>() -> Self
	where
		T: FromKDL<DnD5e> + SystemComponent<System = DnD5e> + 'static + Send + Sync,
	{
		Self {
			add_from_kdl: Box::new(|node, idx, system| {
				let value = T::from_kdl(node, idx, &system)?;
				T::add_component(value, system);
				Ok(())
			}),
		}
	}

	fn add_from_kdl(
		&self,
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		system: &mut DnD5e,
	) -> anyhow::Result<()> {
		(*self.add_from_kdl)(node, value_idx, system)?;
		Ok(())
	}
}

pub trait KDLNode {
	fn id() -> &'static str
	where
		Self: Sized;
}

pub trait FromKDL<System> {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		system: &System,
	) -> anyhow::Result<Self>
	where
		Self: Sized;
}
macro_rules! impl_fromkdl {
	($target:ty, $method:ident, $map:expr) => {
		impl<S> FromKDL<S> for $target {
			fn from_kdl(
				node: &kdl::KdlNode,
				value_idx: &mut ValueIdx,
				_: &S,
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
impl<S> FromKDL<S> for String {
	fn from_kdl(node: &kdl::KdlNode, value_idx: &mut ValueIdx, _: &S) -> anyhow::Result<Self> {
		Ok(node.get_str(value_idx.next())?.to_string())
	}
}
impl<T, S> FromKDL<S> for Option<T>
where
	T: FromKDL<S>,
{
	fn from_kdl(node: &kdl::KdlNode, value_idx: &mut ValueIdx, system: &S) -> anyhow::Result<Self> {
		// Instead of consuming the next-idx, just peek to see if there is a value there or not.
		match node.get(**value_idx) {
			Some(_) => T::from_kdl(node, value_idx, system).map(|v| Some(v)),
			None => Ok(None),
		}
	}
}

pub trait SystemComponent {
	type System;

	fn add_component(self, system: &mut Self::System)
	where
		Self: Sized;
}

pub struct FromKDLFactory<T, S> {
	fn_from_kdl:
		Box<dyn Fn(&kdl::KdlNode, &mut ValueIdx, &S) -> anyhow::Result<T> + 'static + Send + Sync>,
}
impl<T, S> FromKDLFactory<T, S> {
	fn new<U>() -> Self
	where
		U: FromKDL<S> + Into<T> + 'static + Send + Sync,
	{
		Self {
			fn_from_kdl: Box::new(|node, idx, system| Ok(U::from_kdl(node, idx, system)?.into())),
		}
	}

	pub fn from_kdl(
		&self,
		node: &kdl::KdlNode,
		idx: &mut ValueIdx,
		system: &S,
	) -> anyhow::Result<T> {
		(*self.fn_from_kdl)(node, idx, system)
	}
}

type BoxAny = Box<dyn Any + 'static + Send + Sync>;
pub struct EvaluatorFactory {
	/// Information about the expected output type of the evaluator.
	/// Used to ensure the expected output type of `from_kdl` matches
	/// that of the registered evaluator, otherwise Any downcast will implode.
	type_info: (TypeId, &'static str),
	eval_type_name: &'static str,
	fn_from_kdl: Box<
		dyn Fn(&kdl::KdlNode, &mut ValueIdx, &DnD5e) -> anyhow::Result<BoxAny>
			+ 'static
			+ Send
			+ Sync,
	>,
}
impl EvaluatorFactory {
	pub fn new<E>() -> Self
	where
		E: Evaluator<Context = Character> + FromKDL<DnD5e> + 'static + Send + Sync,
	{
		Self {
			eval_type_name: std::any::type_name::<E>(),
			type_info: (TypeId::of::<E::Item>(), std::any::type_name::<E::Item>()),
			fn_from_kdl: Box::new(|node, idx, system| {
				let arc_eval: ArcEvaluator<E::Context, E::Item> =
					Arc::new(E::from_kdl(node, idx, system)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<T>(
		&self,
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<GenericEvaluator<Character, T>>
	where
		T: 'static,
	{
		if TypeId::of::<T>() != self.type_info.0 {
			return Err(GeneralError(format!(
				"Incompatible output types: \
				the the evaluator specified by kdl {:?} has the output type {:?}, \
				but the node is expecting an output type of {:?}.",
				self.eval_type_name,
				self.type_info.1,
				std::any::type_name::<T>()
			))
			.into());
		}
		let any = (self.fn_from_kdl)(node, value_idx, system)?;
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
	items: Vec<data::item::Item>,
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
			if let Err(err) = comp_factory.add_from_kdl(node, &mut ValueIdx::default(), self) {
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
		//system.register_mutator::<data::AddProficiency>();
		//system.register_mutator::<data::mutator::AddAbilityScore>();
		//system.register_mutator::<data::mutator::AddArmorClassFormula>();
		//system.register_mutator::<data::mutator::AddAction>();
		//system.register_mutator::<data::mutator::BonusDamage>();
		system.register_mutator::<data::mutator::AddDefense>();
		system.register_mutator::<data::mutator::AddLifeExpectancy>();
		system.register_mutator::<data::mutator::AddMaxHeight>();
		system.register_mutator::<data::mutator::AddMaxHitPoints>();
		system.register_mutator::<data::mutator::AddSavingThrow>();
		system.register_mutator::<data::mutator::AddSavingThrowModifier>();
		system.register_mutator::<data::mutator::AddSkill>();
		system.register_mutator::<data::mutator::AddSkillModifier>();
		system.register_mutator::<data::mutator::AddMaxSpeed>();
		system.register_mutator::<data::mutator::AddMaxSense>();
		system.register_evaluator::<data::evaluator::armor::HasArmorEquipped>();
		system.register_evaluator::<data::evaluator::GetAbilityModifier>();
		//system.register_evaluator::<Any>();
		//system.register_evaluator::<IsProficientWith>();
		//system.register_evaluator::<BySelection<?, ?>>();
		system.register_evaluator::<data::evaluator::GetLevel>();
		system.register_evaluator::<data::evaluator::ByLevel<i64>>();
		system.register_evaluator::<data::evaluator::ByLevel<Option<i64>>>();
		system.register_evaluator::<data::evaluator::ByLevel<String>>();
		system.register_evaluator::<data::evaluator::ByLevel<Option<String>>>();
		system
	}

	pub fn register_component<T>(&mut self)
	where
		T: FromKDL<Self> + KDLNode + SystemComponent<System = Self> + 'static + Send + Sync,
	{
		assert!(!self.root_registry.contains_key(T::id()));
		self.root_registry
			.insert(T::id(), ComponentFactory::new::<T>().into());
	}

	pub fn register_mutator<T>(&mut self)
	where
		T: Mutator<Target = data::character::Character>
			+ KDLNode
			+ FromKDL<DnD5e>
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
		E: Evaluator<Context = Character> + KDLNode + FromKDL<DnD5e> + 'static + Send + Sync,
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

#[cfg(test)]
impl DnD5e {
	pub fn default_with_eval<E>() -> Self
	where
		E: Evaluator<Context = Character> + KDLNode + FromKDL<DnD5e> + 'static + Send + Sync,
	{
		let mut system = Self::default();
		system.register_evaluator::<E>();
		system
	}

	pub fn default_with_mut<M>() -> Self
	where
		M: Mutator<Target = data::character::Character>
			+ KDLNode
			+ FromKDL<DnD5e>
			+ 'static
			+ Send
			+ Sync,
	{
		let mut system = Self::default();
		system.register_mutator::<M>();
		system
	}

	pub fn parse_kdl_evaluator<T>(
		&self,
		doc: &str,
	) -> anyhow::Result<GenericEvaluator<Character, T>>
	where
		T: 'static,
	{
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document
			.query("evaluator")?
			.expect("missing evaluator node");
		let mut idx = ValueIdx::default();
		let factory = self.get_evaluator_factory(node.get_str(idx.next())?)?;
		factory.from_kdl::<T>(node, &mut idx, &self)
	}

	pub fn parse_kdl_mutator(&self, doc: &str) -> anyhow::Result<BoxedMutator> {
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document.query("mutator")?.expect("missing mutator node");
		let mut idx = ValueIdx::default();
		let factory = self.get_mutator_factory(node.get_str(idx.next())?)?;
		factory.from_kdl(node, &mut idx, &self)
	}

	pub fn defaulteval_parse_kdl<E>(
		doc: &str,
	) -> anyhow::Result<GenericEvaluator<Character, E::Item>>
	where
		E: Evaluator<Context = Character> + KDLNode + FromKDL<DnD5e> + 'static + Send + Sync,
		E::Item: 'static,
	{
		Self::default_with_eval::<E>().parse_kdl_evaluator::<E::Item>(doc)
	}

	pub fn defaultmut_parse_kdl<M>(doc: &str) -> anyhow::Result<BoxedMutator>
	where
		M: Mutator<Target = data::character::Character>
			+ KDLNode
			+ FromKDL<DnD5e>
			+ 'static
			+ Send
			+ Sync,
	{
		Self::default_with_mut::<M>().parse_kdl_mutator(doc)
	}
}
