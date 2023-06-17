use crate::{
	kdl_ext::{AsKdl, FromKDL, KDLNode, NodeContext},
	utility::{ArcEvaluator, ArcMutator, Evaluator, GenericEvaluator, GenericMutator, Mutator},
};
use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::Arc,
};

#[derive(Clone)]
pub struct ArcNodeRegistry(Arc<NodeRegistry>);
impl From<NodeRegistry> for ArcNodeRegistry {
	fn from(value: NodeRegistry) -> Self {
		Self(Arc::new(value))
	}
}
impl ArcNodeRegistry {
	pub fn arc(&self) -> &Arc<NodeRegistry> {
		&self.0
	}
}
impl std::ops::Deref for ArcNodeRegistry {
	type Target = NodeRegistry;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl PartialEq for ArcNodeRegistry {
	fn eq(&self, other: &Self) -> bool {
		Arc::ptr_eq(&self.0, &other.0)
	}
}

pub struct NodeRegistry {
	mutators: HashMap<&'static str, MutatorFactory>,
	evaluators: HashMap<&'static str, EvaluatorFactory>,
}

impl Default for NodeRegistry {
	fn default() -> Self {
		Self {
			mutators: HashMap::new(),
			evaluators: HashMap::new(),
		}
	}
}

impl NodeRegistry {
	pub fn register_mutator<T>(&mut self)
	where
		T: Mutator + KDLNode + FromKDL + 'static + Send + Sync,
	{
		assert!(!self.mutators.contains_key(T::id()));
		self.mutators.insert(T::id(), MutatorFactory::new::<T>());
	}

	pub fn register_evaluator<E>(&mut self)
	where
		E: Evaluator + KDLNode + FromKDL + 'static + Send + Sync,
	{
		assert!(!self.mutators.contains_key(E::id()));
		self.evaluators
			.insert(E::id(), EvaluatorFactory::new::<E>());
	}

	pub fn get_mutator_factory(&self, id: &str) -> anyhow::Result<&MutatorFactory> {
		self.mutators
			.get(id)
			.ok_or(MissingRegistration("mutator", id.to_owned()).into())
	}

	pub fn get_evaluator_factory(&self, id: &str) -> anyhow::Result<&EvaluatorFactory> {
		self.evaluators
			.get(id)
			.ok_or(MissingRegistration("evaluator", id.to_owned()).into())
	}
}

#[derive(thiserror::Error, Debug)]
#[error("No {0} registration found for {1:?}.")]
struct MissingRegistration(&'static str, String);

type BoxAny = Box<dyn Any + 'static + Send + Sync>;

pub struct MutatorFactory {
	type_name: &'static str,
	target_type_info: (TypeId, &'static str),
	fn_from_kdl: Box<
		dyn Fn(&kdl::KdlNode, &mut NodeContext) -> anyhow::Result<BoxAny> + 'static + Send + Sync,
	>,
}

impl MutatorFactory {
	pub fn new<M>() -> Self
	where
		M: Mutator + FromKDL + 'static + Send + Sync,
	{
		Self {
			type_name: std::any::type_name::<M>(),
			target_type_info: (
				TypeId::of::<M::Target>(),
				std::any::type_name::<M::Target>(),
			),
			fn_from_kdl: Box::new(|node, ctx| {
				let arc_eval: ArcMutator<M::Target> = Arc::new(M::from_kdl(node, ctx)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<T>(
		&self,
		node: &kdl::KdlNode,
		ctx: &mut NodeContext,
	) -> anyhow::Result<GenericMutator<T>>
	where
		T: 'static,
	{
		if TypeId::of::<T>() != self.target_type_info.0 {
			return Err(IncompatibleTypes(
				"target",
				self.type_name,
				self.target_type_info.1,
				std::any::type_name::<T>(),
			)
			.into());
		}
		let any = (self.fn_from_kdl)(node, ctx)?;
		let eval = any
			.downcast::<ArcMutator<T>>()
			.expect("failed to unpack boxed arc-evaluator");
		Ok(GenericMutator::new(*eval))
	}
}

pub struct EvaluatorFactory {
	type_name: &'static str,
	/// Information about the expected output type of the evaluator.
	/// Used to ensure the expected output type of `from_kdl` matches
	/// that of the registered evaluator, otherwise Any downcast will implode.
	item_type_info: (TypeId, &'static str),
	ctx_type_info: (TypeId, &'static str),
	fn_from_kdl: Box<
		dyn Fn(&kdl::KdlNode, &mut NodeContext) -> anyhow::Result<BoxAny> + 'static + Send + Sync,
	>,
}

impl EvaluatorFactory {
	pub fn new<E>() -> Self
	where
		E: Evaluator + FromKDL + 'static + Send + Sync,
	{
		Self {
			type_name: std::any::type_name::<E>(),
			ctx_type_info: (
				TypeId::of::<E::Context>(),
				std::any::type_name::<E::Context>(),
			),
			item_type_info: (TypeId::of::<E::Item>(), std::any::type_name::<E::Item>()),
			fn_from_kdl: Box::new(|node, ctx| {
				let arc_eval: ArcEvaluator<E::Context, E::Item> = Arc::new(E::from_kdl(node, ctx)?);
				Ok(Box::new(arc_eval))
			}),
		}
	}

	pub fn from_kdl<C, T>(
		&self,
		node: &kdl::KdlNode,
		ctx: &mut NodeContext,
	) -> anyhow::Result<GenericEvaluator<C, T>>
	where
		C: 'static,
		T: 'static,
	{
		if TypeId::of::<C>() != self.ctx_type_info.0 {
			return Err(IncompatibleTypes(
				"context",
				self.type_name,
				self.ctx_type_info.1,
				std::any::type_name::<C>(),
			)
			.into());
		}

		if TypeId::of::<T>() != self.item_type_info.0 {
			return Err(IncompatibleTypes(
				"output",
				self.type_name,
				self.item_type_info.1,
				std::any::type_name::<T>(),
			)
			.into());
		}

		let any = (self.fn_from_kdl)(node, ctx)?;
		let eval = any
			.downcast::<ArcEvaluator<C, T>>()
			.expect("failed to unpack boxed arc-evaluator");
		Ok(GenericEvaluator::new(*eval))
	}
}

#[derive(thiserror::Error, Debug)]
#[error(
	"Incompatible {0} types: \
	the evaluator specified by kdl {1} has the {0} type {2}, \
	but the node is expecting an {0} type of {3}."
)]
struct IncompatibleTypes(&'static str, &'static str, &'static str, &'static str);

#[cfg(test)]
impl NodeRegistry {
	pub fn default_with_eval<E>() -> Self
	where
		E: Evaluator + KDLNode + FromKDL + 'static + Send + Sync,
	{
		let mut node_reg = Self::default();
		node_reg.register_evaluator::<E>();
		node_reg
	}

	pub fn default_with_mut<M>() -> Self
	where
		M: Mutator + KDLNode + FromKDL + 'static + Send + Sync,
	{
		let mut node_reg = Self::default();
		node_reg.register_mutator::<M>();
		node_reg
	}

	pub fn parse_kdl_evaluator<C, T>(self, doc: &str) -> anyhow::Result<GenericEvaluator<C, T>>
	where
		C: 'static,
		T: 'static,
	{
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document
			.query("scope() > evaluator")?
			.expect("missing evaluator node");
		NodeContext::registry(self).parse_evaluator::<C, T>(node)
	}

	pub fn parse_kdl_mutator<T>(self, doc: &str) -> anyhow::Result<GenericMutator<T>>
	where
		T: 'static,
	{
		let document = doc.parse::<kdl::KdlDocument>()?;
		let node = document
			.query("scope() > mutator")?
			.expect("missing mutator node");
		NodeContext::registry(self).parse_mutator::<T>(node)
	}

	pub fn defaulteval_parse_kdl<E>(
		doc: &str,
	) -> anyhow::Result<GenericEvaluator<E::Context, E::Item>>
	where
		E: Evaluator + KDLNode + FromKDL + 'static + Send + Sync,
		E::Item: 'static,
	{
		Self::default_with_eval::<E>().parse_kdl_evaluator::<E::Context, E::Item>(doc)
	}

	pub fn defaultmut_parse_kdl<M>(doc: &str) -> anyhow::Result<GenericMutator<M::Target>>
	where
		M: Mutator + KDLNode + FromKDL + 'static + Send + Sync,
		M::Target: 'static,
	{
		Self::default_with_mut::<M>().parse_kdl_mutator(doc)
	}
}
