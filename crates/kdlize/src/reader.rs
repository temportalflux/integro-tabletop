use std::str::FromStr;
use crate::{error::{Error, MissingEntryValue, InvalidValueType, InvalidQueryFormat}, FromKdl, ext::{EntryExt, NodeExt, DocumentQueryExt, DocumentExt}};

pub struct NodeReader<'doc, Context> {
	node: &'doc kdl::KdlNode,
	ctx: Context,
	index_cursor: usize,
	is_root: bool,
}

impl<'doc, Context> ToString for NodeReader<'doc, Context> {
	fn to_string(&self) -> String {
		self.node.to_string()
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
	pub fn new_root(node: &'doc kdl::KdlNode, ctx: Context) -> Self {
		Self {
			node,
			ctx,
			index_cursor: 0,
			is_root: true,
		}
	}

	pub fn new_child(node: &'doc kdl::KdlNode, ctx: Context) -> Self {
		Self {
			node,
			ctx,
			index_cursor: 0,
			is_root: false,
		}
	}

	fn next_node(&self, node: &'doc kdl::KdlNode) -> Self where Context: Clone {
		Self::new_child(node, self.ctx.clone())
	}

	pub fn is_root(&self) -> bool {
		self.is_root
	}

	pub fn context(&self) -> &Context {
		&self.ctx
	}

	pub fn name(&self) -> &kdl::KdlIdentifier {
		self.node.name()
	}

	pub fn entries(&self) -> &[kdl::KdlEntry] {
		self.node.entries()
	}

	pub fn children(&self) -> Option<Vec<Self>> where Context: Clone {
		let Some(doc) = self.node.children() else { return None; };
		Some(Self::iter_from(&self.ctx, doc.nodes().iter()))
	}

	fn iter_from(
		ctx: &Context,
		iter: impl Iterator<Item = &'doc kdl::KdlNode> + 'doc,
	) -> Vec<Self> where Context: Clone {
		iter.map(|node| Self::new_child(node, ctx.clone()))
			.collect()
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
	fn peak_idx(&self) -> usize {
		self.index_cursor
	}

	pub fn peak_opt(&self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(self.peak_idx())
	}
	pub fn peak_req(&self) -> Result<&'doc kdl::KdlEntry, MissingEntryValue> {
		self.node.entry_req(self.peak_idx())
	}
	pub fn peak_bool_opt(&self) -> Result<Option<bool>, InvalidValueType> {
		self.node.get_bool_opt(self.peak_idx())
	}
	pub fn peak_i64_opt(&self) -> Result<Option<i64>, InvalidValueType> {
		self.node.get_i64_opt(self.peak_idx())
	}
	pub fn peak_f64_opt(&self) -> Result<Option<f64>, InvalidValueType> {
		self.node.get_f64_opt(self.peak_idx())
	}
	pub fn peak_str_opt(&self) -> Result<Option<&'doc str>, InvalidValueType> {
		self.node.get_str_opt(self.peak_idx())
	}
	pub fn peak_bool_req(&self) -> Result<bool, Error> {
		self.node.get_bool_req(self.peak_idx())
	}
	pub fn peak_i64_req(&self) -> Result<i64, Error> {
		self.node.get_i64_req(self.peak_idx())
	}
	pub fn peak_f64_req(&self) -> Result<f64, Error> {
		self.node.get_f64_req(self.peak_idx())
	}
	pub fn peak_str_req(&self) -> Result<&'doc str, Error> {
		self.node.get_str_req(self.peak_idx())
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
	fn consume_idx(&mut self) -> usize {
		let consumed = self.index_cursor;
		self.index_cursor += 1;
		consumed
	}

	pub fn next_opt(&mut self) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(self.consume_idx())
	}
	pub fn next_req(&mut self) -> Result<&'doc kdl::KdlEntry, MissingEntryValue> {
		self.node.entry_req(self.consume_idx())
	}
	pub fn next_bool_opt(&mut self) -> Result<Option<bool>, InvalidValueType> {
		self.node.get_bool_opt(self.consume_idx())
	}
	pub fn next_i64_opt(&mut self) -> Result<Option<i64>, InvalidValueType> {
		self.node.get_i64_opt(self.consume_idx())
	}
	pub fn next_f64_opt(&mut self) -> Result<Option<f64>, InvalidValueType> {
		self.node.get_f64_opt(self.consume_idx())
	}
	pub fn next_str_opt(&mut self) -> Result<Option<&'doc str>, InvalidValueType> {
		self.node.get_str_opt(self.consume_idx())
	}
	pub fn next_bool_req(&mut self) -> Result<bool, Error> {
		self.node.get_bool_req(self.consume_idx())
	}
	pub fn next_i64_req(&mut self) -> Result<i64, Error> {
		self.node.get_i64_req(self.consume_idx())
	}
	pub fn next_f64_req(&mut self) -> Result<f64, Error> {
		self.node.get_f64_req(self.consume_idx())
	}
	pub fn next_str_req(&mut self) -> Result<&'doc str, Error> {
		self.node.get_str_req(self.consume_idx())
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
	pub fn get_opt(&self, key: impl AsRef<str>) -> Option<&'doc kdl::KdlEntry> {
		self.node.entry_opt(key.as_ref())
	}
	pub fn get_req(
		&self,
		key: impl AsRef<str>,
	) -> Result<&'doc kdl::KdlEntry, MissingEntryValue> {
		self.node.entry_req(key.as_ref())
	}
	pub fn get_bool_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<bool>, InvalidValueType> {
		self.node.get_bool_opt(key.as_ref())
	}
	pub fn get_i64_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<i64>, InvalidValueType> {
		self.node.get_i64_opt(key.as_ref())
	}
	pub fn get_f64_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<f64>, InvalidValueType> {
		self.node.get_f64_opt(key.as_ref())
	}
	pub fn get_str_opt(
		&self,
		key: impl AsRef<str>,
	) -> Result<Option<&'doc str>, InvalidValueType> {
		self.node.get_str_opt(key.as_ref())
	}
	pub fn get_bool_req(&self, key: impl AsRef<str>) -> Result<bool, Error> {
		self.node.get_bool_req(key.as_ref())
	}
	pub fn get_i64_req(&self, key: impl AsRef<str>) -> Result<i64, Error> {
		self.node.get_i64_req(key.as_ref())
	}
	pub fn get_f64_req(&self, key: impl AsRef<str>) -> Result<f64, Error> {
		self.node.get_f64_req(key.as_ref())
	}
	pub fn get_str_req(&self, key: impl AsRef<str>) -> Result<&'doc str, Error> {
		self.node.get_str_req(key.as_ref())
	}
}

impl<'doc, Context> NodeReader<'doc, Context> where Context: Clone {
	pub fn query_opt(&self, query: impl AsRef<str>) -> Result<Option<Self>, InvalidQueryFormat> {
		Ok(self.node.query_opt(query)?.map(|node| self.next_node(node)))
	}
	pub fn query_req(&self, query: impl AsRef<str>) -> Result<Self, Error> {
		Ok(self.next_node(self.node.query_req(query)?))
	}
	pub fn query_all(&self, query: impl kdl::IntoKdlQuery) -> Result<Vec<Self>, InvalidQueryFormat> {
		Ok(Self::iter_from(&self.ctx, self.node.query_all(query)?))
	}
	pub fn query_get_all(
		&self,
		query: impl kdl::IntoKdlQuery,
		key: impl Into<kdl::NodeKey>,
	) -> Result<impl Iterator<Item = &kdl::KdlValue>, InvalidQueryFormat> {
		self.node.query_get_all(query, key).map_err(|e| InvalidQueryFormat(e))
	}
}

impl<'doc, Context> NodeReader<'doc, Context> {
	pub fn peak_type_req(&self) -> Result<&str, Error> {
		Ok(self.peak_req()?.type_req()?)
	}

	pub fn next_str_opt_t<T>(&mut self) -> Result<Option<T>, anyhow::Error>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		let Some(str) = self.next_str_opt()? else { return Ok(None); };
		Ok(Some(T::from_str(str)?))
	}

	pub fn next_str_req_t<T>(&mut self) -> Result<T, anyhow::Error>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		Ok(T::from_str(self.next_str_req()?)?)
	}

	pub fn get_str_req_t<T>(&self, key: impl AsRef<str>) -> Result<T, anyhow::Error>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		Ok(T::from_str(self.get_str_req(key)?)?)
	}

	pub fn query_opt_t<T>(&self, query: impl AsRef<str>) -> Result<Option<T>, anyhow::Error>
	where T: FromKdl<Context>, anyhow::Error: From<T::Error>, Context: Clone {
		let Some(mut node) = self.query_opt(query).map_err(Error::from)? else { return Ok(None); };
		Ok(Some(T::from_kdl(&mut node)?))
	}

	pub fn query_req_t<T>(&self, query: impl AsRef<str>) -> Result<T, anyhow::Error>
	where T: FromKdl<Context>, anyhow::Error: From<T::Error>, Context: Clone {
		Ok(T::from_kdl(&mut self.query_req(query)?)?)
	}

	pub fn query_all_t<T>(&self, query: impl kdl::IntoKdlQuery) -> Result<Vec<T>, anyhow::Error>
	where T: FromKdl<Context>, anyhow::Error: From<T::Error>, Context: Clone {
		let nodes = self.query_all(query).map_err(Error::from)?;
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
	) -> Result<Option<T>, anyhow::Error>
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
	) -> Result<T, anyhow::Error>
	where
		T: FromStr,
		T::Err: std::error::Error + Send + Sync + 'static,
	{
		Ok(T::from_str(self.query_str_req(query, key)?)?)
	}
}

impl<'doc, Context> DocumentExt for NodeReader<'doc, Context> {
	fn query_bool_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<bool>, Error> {
		self.node.query_bool_opt(query, key)
	}
	fn query_i64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<i64>, Error> {
		self.node.query_i64_opt(query, key)
	}
	fn query_f64_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<f64>, Error> {
		self.node.query_f64_opt(query, key)
	}
	fn query_str_opt(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Option<&str>, Error> {
		self.node.query_str_opt(query, key)
	}

	fn query_bool_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<bool, Error> {
		self.node.query_bool_req(query, key)
	}
	fn query_i64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<i64, Error> {
		self.node.query_i64_req(query, key)
	}
	fn query_f64_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<f64, Error> {
		self.node.query_f64_req(query, key)
	}
	fn query_str_req(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<&str, Error> {
		self.node.query_str_req(query, key)
	}

	fn query_bool_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<bool>, Error> {
		self.node.query_bool_all(query, key)
	}
	fn query_i64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<i64>, Error> {
		self.node.query_i64_all(query, key)
	}
	fn query_f64_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<f64>, Error> {
		self.node.query_f64_all(query, key)
	}
	fn query_str_all(
		&self,
		query: impl AsRef<str>,
		key: impl Into<kdl::NodeKey>,
	) -> Result<Vec<&str>, Error> {
		self.node.query_str_all(query, key)
	}
}
