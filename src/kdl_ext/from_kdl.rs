use super::{DocumentExt, DocumentQueryExt, EntryExt, NodeExt};
use crate::system::core::{NodeRegistry, SourceId};
use std::{str::FromStr, sync::Arc};

#[derive(Default, Clone)]
pub struct NodeContext {
	root_id: Arc<SourceId>,
	node_registry: Arc<NodeRegistry>,
}

impl NodeContext {
	pub fn new(id: Arc<SourceId>, registry: Arc<NodeRegistry>) -> Self {
		Self {
			root_id: id,
			node_registry: registry,
		}
	}

	#[cfg(test)]
	pub fn registry(registry: NodeRegistry) -> Self {
		Self {
			node_registry: Arc::new(registry),
			..Default::default()
		}
	}

	pub fn id(&self) -> &SourceId {
		&*self.root_id
	}

	pub fn node_reg(&self) -> &Arc<NodeRegistry> {
		&self.node_registry
	}
}
pub struct NodeReader<'doc> {
	node: &'doc kdl::KdlNode,
	ctx: NodeContext,
	index_cursor: usize,
	is_root: bool,
}
impl<'doc> ToString for NodeReader<'doc> {
	fn to_string(&self) -> String {
		self.node.to_string()
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn new_root(node: &'doc kdl::KdlNode, ctx: NodeContext) -> Self {
		Self {
			node,
			ctx,
			index_cursor: 0,
			is_root: true,
		}
	}

	pub fn new_child(node: &'doc kdl::KdlNode, ctx: NodeContext) -> Self {
		Self {
			node,
			ctx,
			index_cursor: 0,
			is_root: false,
		}
	}

	fn next_node(&self, node: &'doc kdl::KdlNode) -> Self {
		Self::new_child(node, self.ctx.clone())
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

	pub fn children(&self) -> Option<Vec<Self>> {
		let Some(doc) = self.node.children() else { return None; };
		Some(Self::iter_from(&self.ctx, doc.nodes().iter()))
	}

	fn iter_from(
		ctx: &NodeContext,
		iter: impl Iterator<Item = &'doc kdl::KdlNode> + 'doc,
	) -> Vec<Self> {
		iter.map(|node| Self::new_child(node, ctx.clone()))
			.collect()
	}
}
impl<'doc> NodeReader<'doc> {
	fn peak_idx(&self) -> usize {
		self.index_cursor
	}

	pub fn peak_opt(&self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(self.peak_idx())
	}
	pub fn peak_req(&self) -> Result<&'doc kdl::KdlEntry, super::EntryMissing> {
		self.node.entry_req(self.peak_idx())
	}
	pub fn peak_bool_opt(&self) -> Result<Option<bool>, super::InvalidValueType> {
		self.node.get_bool_opt(self.peak_idx())
	}
	pub fn peak_i64_opt(&self) -> Result<Option<i64>, super::InvalidValueType> {
		self.node.get_i64_opt(self.peak_idx())
	}
	pub fn peak_f64_opt(&self) -> Result<Option<f64>, super::InvalidValueType> {
		self.node.get_f64_opt(self.peak_idx())
	}
	pub fn peak_str_opt(&self) -> Result<Option<&'doc str>, super::InvalidValueType> {
		self.node.get_str_opt(self.peak_idx())
	}
	pub fn peak_bool_req(&self) -> Result<bool, super::RequiredValue> {
		self.node.get_bool_req(self.peak_idx())
	}
	pub fn peak_i64_req(&self) -> Result<i64, super::RequiredValue> {
		self.node.get_i64_req(self.peak_idx())
	}
	pub fn peak_f64_req(&self) -> Result<f64, super::RequiredValue> {
		self.node.get_f64_req(self.peak_idx())
	}
	pub fn peak_str_req(&self) -> Result<&'doc str, super::RequiredValue> {
		self.node.get_str_req(self.peak_idx())
	}
}
impl<'doc> NodeReader<'doc> {
	fn consume_idx(&mut self) -> usize {
		let consumed = self.index_cursor;
		self.index_cursor += 1;
		consumed
	}

	pub fn next_opt(&mut self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(self.consume_idx())
	}
	pub fn next_req(&mut self) -> Result<&'doc kdl::KdlEntry, super::EntryMissing> {
		self.node.entry_req(self.consume_idx())
	}
	pub fn next_bool_opt(&mut self) -> Result<Option<bool>, super::InvalidValueType> {
		self.node.get_bool_opt(self.consume_idx())
	}
	pub fn next_i64_opt(&mut self) -> Result<Option<i64>, super::InvalidValueType> {
		self.node.get_i64_opt(self.consume_idx())
	}
	pub fn next_f64_opt(&mut self) -> Result<Option<f64>, super::InvalidValueType> {
		self.node.get_f64_opt(self.consume_idx())
	}
	pub fn next_str_opt(&mut self) -> Result<Option<&'doc str>, super::InvalidValueType> {
		self.node.get_str_opt(self.consume_idx())
	}
	pub fn next_bool_req(&mut self) -> Result<bool, super::RequiredValue> {
		self.node.get_bool_req(self.consume_idx())
	}
	pub fn next_i64_req(&mut self) -> Result<i64, super::RequiredValue> {
		self.node.get_i64_req(self.consume_idx())
	}
	pub fn next_f64_req(&mut self) -> Result<f64, super::RequiredValue> {
		self.node.get_f64_req(self.consume_idx())
	}
	pub fn next_str_req(&mut self) -> Result<&'doc str, super::RequiredValue> {
		self.node.get_str_req(self.consume_idx())
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
	pub fn query_all(&self, query: impl kdl::IntoKdlQuery) -> Result<Vec<Self>, kdl::KdlError> {
		Ok(Self::iter_from(&self.ctx, self.node.query_all(query)?))
	}
	pub fn query_get_all(
		&self,
		query: impl kdl::IntoKdlQuery,
		key: impl Into<kdl::NodeKey>,
	) -> Result<impl Iterator<Item = &kdl::KdlValue>, kdl::KdlError> {
		self.node.query_get_all(query, key)
	}
}
impl<'doc> NodeReader<'doc> {
	pub fn peak_type_req(&self) -> anyhow::Result<&str> {
		Ok(self.peak_req()?.type_req()?)
	}

	pub fn next_str_opt_t<T>(&mut self) -> anyhow::Result<Option<T>>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		let Some(str) = self.next_str_opt()? else { return Ok(None); };
		Ok(Some(T::from_str(str)?))
	}

	pub fn next_str_req_t<T>(&mut self) -> anyhow::Result<T>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		Ok(T::from_str(self.next_str_req()?)?)
	}

	pub fn get_str_req_t<T>(&self, key: impl AsRef<str>) -> anyhow::Result<T>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		Ok(T::from_str(self.get_str_req(key)?)?)
	}

	pub fn query_opt_t<T: FromKDL>(&self, query: impl AsRef<str>) -> anyhow::Result<Option<T>> {
		let Some(mut node) = self.query_opt(query)? else { return Ok(None); };
		Ok(Some(T::from_kdl(&mut node)?))
	}

	pub fn query_req_t<T: FromKDL>(&self, query: impl AsRef<str>) -> anyhow::Result<T> {
		Ok(T::from_kdl(&mut self.query_req(query)?)?)
	}

	pub fn query_all_t<T: FromKDL>(&self, query: impl kdl::IntoKdlQuery) -> anyhow::Result<Vec<T>> {
		let nodes = self.query_all(query)?;
		let mut vec = Vec::with_capacity(nodes.len());
		for mut node in nodes {
			vec.push(T::from_kdl(&mut node)?);
		}
		Ok(vec)
	}

	pub fn query_str_opt_t<T>(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> anyhow::Result<Option<T>>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		let Some(str) = self.query_str_opt(query, key)? else { return Ok(None); };
		Ok(Some(T::from_str(str)?))
	}

	pub fn query_str_req_t<T>(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> anyhow::Result<T>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		Ok(T::from_str(self.query_str_req(query, key)?)?)
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
impl<'doc> NodeReader<'doc> {
	pub fn node_reg(&self) -> &Arc<NodeRegistry> {
		self.ctx.node_reg()
	}

	pub fn query_source_opt(&self) -> anyhow::Result<Option<SourceId>> {
		match self.query_str_opt("scope() > source", 0)? {
			Some(id_str) => Ok(Some(
				SourceId::from_str(id_str)?.with_basis(self.id(), true),
			)),
			None if self.is_root => Ok(Some(self.id().clone().with_basis(self.id(), true))),
			None => Ok(None),
		}
	}

	pub fn query_source_req(&self) -> anyhow::Result<SourceId> {
		Ok(self.query_source_opt()?.ok_or(MissingSource)?)
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
