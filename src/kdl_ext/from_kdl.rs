use super::{DocumentExt, DocumentQueryExt, NodeExt};
use crate::{
	system::core::{NodeRegistry, SourceId},
	utility::{GenericEvaluator, GenericMutator},
};
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct NodeContext {
	root_id: Arc<SourceId>,
	index_cursor: usize,
	node_registry: Arc<NodeRegistry>,
	inheiret_source: bool,
}

impl NodeContext {
	pub fn new(id: Arc<SourceId>, registry: Arc<NodeRegistry>) -> Self {
		Self {
			root_id: id,
			node_registry: registry,
			index_cursor: 0,
			inheiret_source: true,
		}
	}

	#[cfg(test)]
	pub fn registry(registry: NodeRegistry) -> Self {
		Self {
			node_registry: Arc::new(registry),
			..Default::default()
		}
	}

	pub fn inheiret_source(mut self, inheiret: bool) -> Self {
		self.set_inheiret_source(inheiret);
		self
	}

	pub fn set_inheiret_source(&mut self, inheiret: bool) {
		self.inheiret_source = inheiret;
	}

	pub fn id(&self) -> &SourceId {
		&*self.root_id
	}

	pub fn peak_idx(&self) -> usize {
		self.index_cursor
	}

	pub fn consume_idx(&mut self) -> usize {
		let consumed = self.index_cursor;
		self.index_cursor += 1;
		consumed
	}

	pub fn next_node(&self) -> Self {
		Self {
			root_id: self.root_id.clone(),
			index_cursor: 0,
			node_registry: self.node_registry.clone(),
			inheiret_source: self.inheiret_source,
		}
	}

	pub fn node_reg(&self) -> &Arc<NodeRegistry> {
		&self.node_registry
	}

	pub fn parse_source_opt(&self, node: &kdl::KdlNode) -> anyhow::Result<Option<SourceId>> {
		use std::str::FromStr;
		match node.query_str_opt("scope() > source", 0)? {
			Some(id_str) => Ok(Some(SourceId::from_str(id_str)?)),
			None if self.inheiret_source => Ok(Some(self.id().clone())),
			None => Ok(None),
		}
	}

	pub fn parse_source_req(&self, node: &kdl::KdlNode) -> anyhow::Result<SourceId> {
		Ok(self.parse_source_opt(node)?.ok_or(MissingSource)?)
	}

	pub fn parse_mutator<'doc, T>(
		&self,
		node: &'doc kdl::KdlNode,
	) -> anyhow::Result<GenericMutator<T>>
	where
		T: 'static,
	{
		let mut ctx = self.next_node();
		let id = node.get_str_req(ctx.consume_idx())?;
		let factory = self.node_registry.get_mutator_factory(id)?;
		factory.from_kdl::<'doc, T>(NodeReader::new(node, ctx))
	}

	pub fn parse_evaluator<C, V>(
		&self,
		node: &kdl::KdlNode,
	) -> anyhow::Result<GenericEvaluator<C, V>>
	where
		C: 'static,
		V: 'static,
	{
		let mut ctx = self.next_node();
		ctx.parse_evaluator_inline(node)
	}

	pub fn parse_evaluator_inline<C, V>(
		&mut self,
		node: &kdl::KdlNode,
	) -> anyhow::Result<GenericEvaluator<C, V>>
	where
		C: 'static,
		V: 'static,
	{
		let id = node.get_str_req(self.consume_idx())?;
		let node_reg = self.node_registry.clone();
		let factory = node_reg.get_evaluator_factory(id)?;
		factory.from_kdl::<C, V>(&mut NodeReader::new(node, self.clone()))
	}
}

pub struct NodeReader<'doc> {
	node: &'doc kdl::KdlNode,
	ctx: NodeContext,
}
impl<'doc> ToString for NodeReader<'doc> {
	fn to_string(&self) -> String {
		self.node.to_string()
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn new(node: &'doc kdl::KdlNode, ctx: NodeContext) -> Self {
		Self { node, ctx }
	}

	pub fn id(&self) -> &SourceId {
		self.ctx.id()
	}

	pub fn name(&self) -> &kdl::KdlIdentifier {
		self.node.name()
	}

	pub fn entries(&self) -> &[kdl::KdlEntry] {
		self.node.entries()
	}

	pub fn children(&self) -> Option<NodeReaderIterator<'doc>> {
		let Some(doc) = self.node.children() else { return None; };
		Some(NodeReaderIterator::<'doc>(
			self.ctx.clone(),
			Box::new(doc.nodes().iter()),
		))
	}

	fn next_node(&self, node: &'doc kdl::KdlNode) -> Self {
		Self::new(node, self.ctx.next_node())
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn peak_opt(&self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(self.ctx.peak_idx())
	}
	pub fn peak_req(&self) -> Result<&'doc kdl::KdlEntry, super::EntryMissing> {
		self.node.entry_req(self.ctx.peak_idx())
	}
	pub fn peak_bool_opt(&self) -> Result<Option<bool>, super::InvalidValueType> {
		self.node.get_bool_opt(self.ctx.peak_idx())
	}
	pub fn peak_i64_opt(&self) -> Result<Option<i64>, super::InvalidValueType> {
		self.node.get_i64_opt(self.ctx.peak_idx())
	}
	pub fn peak_f64_opt(&self) -> Result<Option<f64>, super::InvalidValueType> {
		self.node.get_f64_opt(self.ctx.peak_idx())
	}
	pub fn peak_str_opt(&self) -> Result<Option<&'doc str>, super::InvalidValueType> {
		self.node.get_str_opt(self.ctx.peak_idx())
	}
	pub fn peak_bool_req(&self) -> Result<bool, super::RequiredValue> {
		self.node.get_bool_req(self.ctx.peak_idx())
	}
	pub fn peak_i64_req(&self) -> Result<i64, super::RequiredValue> {
		self.node.get_i64_req(self.ctx.peak_idx())
	}
	pub fn peak_f64_req(&self) -> Result<f64, super::RequiredValue> {
		self.node.get_f64_req(self.ctx.peak_idx())
	}
	pub fn peak_str_req(&self) -> Result<&'doc str, super::RequiredValue> {
		self.node.get_str_req(self.ctx.peak_idx())
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn next_opt(&mut self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(self.ctx.consume_idx())
	}
	pub fn next_req(&mut self) -> Result<&'doc kdl::KdlEntry, super::EntryMissing> {
		self.node.entry_req(self.ctx.consume_idx())
	}
	pub fn next_bool_opt(&mut self) -> Result<Option<bool>, super::InvalidValueType> {
		self.node.get_bool_opt(self.ctx.consume_idx())
	}
	pub fn next_i64_opt(&mut self) -> Result<Option<i64>, super::InvalidValueType> {
		self.node.get_i64_opt(self.ctx.consume_idx())
	}
	pub fn next_f64_opt(&mut self) -> Result<Option<f64>, super::InvalidValueType> {
		self.node.get_f64_opt(self.ctx.consume_idx())
	}
	pub fn next_str_opt(&mut self) -> Result<Option<&'doc str>, super::InvalidValueType> {
		self.node.get_str_opt(self.ctx.consume_idx())
	}
	pub fn next_bool_req(&mut self) -> Result<bool, super::RequiredValue> {
		self.node.get_bool_req(self.ctx.consume_idx())
	}
	pub fn next_i64_req(&mut self) -> Result<i64, super::RequiredValue> {
		self.node.get_i64_req(self.ctx.consume_idx())
	}
	pub fn next_f64_req(&mut self) -> Result<f64, super::RequiredValue> {
		self.node.get_f64_req(self.ctx.consume_idx())
	}
	pub fn next_str_req(&mut self) -> Result<&'doc str, super::RequiredValue> {
		self.node.get_str_req(self.ctx.consume_idx())
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn get_opt(&self, key: impl AsRef<str>) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(key.as_ref())
	}
	pub fn get_req(
		&self,
		key: impl AsRef<str>,
	) -> Result<&'doc kdl::KdlEntry, super::EntryMissing> {
		self.node.entry_req(key.as_ref())
	}
	pub fn get_bool_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<bool>, super::InvalidValueType> {
		self.node.get_bool_opt(key.as_ref())
	}
	pub fn get_i64_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<i64>, super::InvalidValueType> {
		self.node.get_i64_opt(key.as_ref())
	}
	pub fn get_f64_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<f64>, super::InvalidValueType> {
		self.node.get_f64_opt(key.as_ref())
	}
	pub fn get_str_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<&'doc str>, super::InvalidValueType> {
		self.node.get_str_opt(key.as_ref())
	}
	pub fn get_bool_req(&self, key: impl AsRef<str>) -> Result<bool, super::RequiredValue> {
		self.node.get_bool_req(key.as_ref())
	}
	pub fn get_i64_req(&self, key: impl AsRef<str>) -> Result<i64, super::RequiredValue> {
		self.node.get_i64_req(key.as_ref())
	}
	pub fn get_f64_req(&self, key: impl AsRef<str>) -> Result<f64, super::RequiredValue> {
		self.node.get_f64_req(key.as_ref())
	}
	pub fn get_str_req(&self, key: impl AsRef<str>) -> Result<&'doc str, super::RequiredValue> {
		self.node.get_str_req(key.as_ref())
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<Self>, kdl::KdlError> {
		Ok(self.node.query_opt(query)?.map(|node| self.next_node(node)))
	}
	pub fn query_req(&self, query: impl AsRef<str>) -> Result<Self, super::QueryError> {
		Ok(self.next_node(self.node.query_req(query)?))
	}
	pub fn query_all(
		&self,
		query: impl kdl::IntoKdlQuery,
	) -> Result<NodeReaderIterator<'doc>, kdl::KdlError> {
		Ok(NodeReaderIterator::<'doc>(
			self.ctx.clone(),
			Box::new(self.node.query_all(query)?),
		))
	}
	pub fn query_get_all(
		&self,
		query: impl kdl::IntoKdlQuery,
		key: impl Into<kdl::NodeKey>,
	) -> Result<impl Iterator<Item = &kdl::KdlValue>, kdl::KdlError> {
		self.node.query_get_all(query, key)
	}
}
impl<'doc> DocumentExt for NodeReader<'doc> {
	fn query_bool_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, super::QueryError> {
		self.node.query_bool_opt(query, key)
	}
	fn query_i64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<i64>, super::QueryError> {
		self.node.query_i64_opt(query, key)
	}
	fn query_f64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<f64>, super::QueryError> {
		self.node.query_f64_opt(query, key)
	}
	fn query_str_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<&str>, super::QueryError> {
		self.node.query_str_opt(query, key)
	}

	fn query_bool_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<bool, super::QueryError> {
		self.node.query_bool_req(query, key)
	}
	fn query_i64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<i64, super::QueryError> {
		self.node.query_i64_req(query, key)
	}
	fn query_f64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<f64, super::QueryError> {
		self.node.query_f64_req(query, key)
	}
	fn query_str_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<&str, super::QueryError> {
		self.node.query_str_req(query, key)
	}

	fn query_bool_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<bool>, super::QueryError> {
		self.node.query_bool_all(query, key)
	}
	fn query_i64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<i64>, super::QueryError> {
		self.node.query_i64_all(query, key)
	}
	fn query_f64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<f64>, super::QueryError> {
		self.node.query_f64_all(query, key)
	}
	fn query_str_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<&str>, super::QueryError> {
		self.node.query_str_all(query, key)
	}
}
pub struct NodeReaderIterator<'doc>(
	NodeContext,
	Box<dyn Iterator<Item = &'doc kdl::KdlNode> + 'doc>,
);
impl<'doc> Iterator for NodeReaderIterator<'doc> {
	type Item = NodeReader<'doc>;

	fn next(&mut self) -> Option<Self::Item> {
		self.1.next().map(|node| NodeReader {
			node,
			ctx: self.0.next_node(),
		})
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn node_reg(&self) -> &Arc<NodeRegistry> {
		self.ctx.node_reg()
	}

	pub fn set_inheiret_source(&mut self, inheiret: bool) {
		self.ctx.set_inheiret_source(inheiret);
	}

	pub fn parse_source_opt(&self) -> anyhow::Result<Option<SourceId>> {
		self.ctx.parse_source_opt(self.node)
	}

	pub fn parse_source_req(&self) -> anyhow::Result<SourceId> {
		self.ctx.parse_source_req(self.node)
	}

	pub fn parse_mutator<T>(&self) -> anyhow::Result<GenericMutator<T>>
	where
		T: 'static,
	{
		self.ctx.parse_mutator(self.node)
	}

	pub fn parse_evaluator<C, V>(&self) -> anyhow::Result<GenericEvaluator<C, V>>
	where
		C: 'static,
		V: 'static,
	{
		self.ctx.parse_evaluator(self.node)
	}

	pub fn parse_evaluator_inline<C, V>(&mut self) -> anyhow::Result<GenericEvaluator<C, V>>
	where
		C: 'static,
		V: 'static,
	{
		self.ctx.parse_evaluator_inline(self.node)
	}
}

#[derive(thiserror::Error, Debug)]
#[error("Missing source field")]
pub struct MissingSource;

pub trait KDLNode {
	fn id() -> &'static str
	where
		Self: Sized;

	fn get_id(&self) -> &'static str;
}

#[macro_export]
macro_rules! impl_kdl_node {
	($target:ty, $id:expr) => {
		impl crate::kdl_ext::KDLNode for $target {
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
	fn from_kdl<'doc>(node: &mut NodeReader<'doc>) -> anyhow::Result<Self>
	where
		Self: Sized;
}

macro_rules! impl_fromkdl {
	($target:ty, $method:ident, $map:expr) => {
		impl FromKDL for $target {
			fn from_kdl<'doc>(node: &mut NodeReader<'doc>) -> anyhow::Result<Self> {
				Ok(node.$method().map($map)?)
			}
		}
	};
}
impl_fromkdl!(bool, next_bool_req, |v| v);
impl_fromkdl!(u8, next_i64_req, |v| v as u8);
impl_fromkdl!(i8, next_i64_req, |v| v as i8);
impl_fromkdl!(u16, next_i64_req, |v| v as u16);
impl_fromkdl!(i16, next_i64_req, |v| v as i16);
impl_fromkdl!(u32, next_i64_req, |v| v as u32);
impl_fromkdl!(i32, next_i64_req, |v| v as i32);
impl_fromkdl!(u64, next_i64_req, |v| v as u64);
impl_fromkdl!(i64, next_i64_req, |v| v);
impl_fromkdl!(u128, next_i64_req, |v| v as u128);
impl_fromkdl!(i128, next_i64_req, |v| v as i128);
impl_fromkdl!(usize, next_i64_req, |v| v as usize);
impl_fromkdl!(isize, next_i64_req, |v| v as isize);
impl_fromkdl!(f32, next_f64_req, |v| v as f32);
impl_fromkdl!(f64, next_f64_req, |v| v);
impl FromKDL for String {
	fn from_kdl<'doc>(node: &mut NodeReader<'doc>) -> anyhow::Result<Self> {
		Ok(node.next_str_req()?.to_string())
	}
}

impl<T> FromKDL for Option<T>
where
	T: FromKDL,
{
	fn from_kdl<'doc>(node: &mut NodeReader<'doc>) -> anyhow::Result<Self> {
		// Instead of consuming the next-idx, just peek to see if there is a value there or not.
		match node.peak_opt() {
			Some(_) => T::from_kdl(node).map(|v| Some(v)),
			None => Ok(None),
		}
	}
}
