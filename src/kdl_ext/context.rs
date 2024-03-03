use crate::system::{generics, SourceId};
use kdlize::{ext::DocumentExt, NodeReader};
use std::{str::FromStr, sync::Arc};

#[derive(thiserror::Error, Debug)]
#[error("Missing source field")]
pub struct MissingSource;

#[derive(Default, Clone)]
pub struct NodeContext {
	root_id: Arc<SourceId>,
	node_registry: Arc<generics::Registry>,
}

impl NodeContext {
	pub fn new(id: Arc<SourceId>, registry: Arc<generics::Registry>) -> Self {
		Self {
			root_id: id,
			node_registry: registry,
		}
	}

	#[cfg(test)]
	pub fn registry(registry: generics::Registry) -> Self {
		Self {
			node_registry: Arc::new(registry),
			..Default::default()
		}
	}

	pub fn id(&self) -> &SourceId {
		&*self.root_id
	}

	pub fn node_reg(&self) -> &Arc<generics::Registry> {
		&self.node_registry
	}
}

pub fn query_source_opt<'doc>(reader: &NodeReader<'doc, NodeContext>) -> anyhow::Result<Option<SourceId>> {
	match reader.query_str_opt("scope() > source", 0)? {
		Some(id_str) => Ok(Some(
			SourceId::from_str(id_str)?.with_relative_basis(reader.context().id(), true),
		)),
		None if reader.is_root() => {
			let id = reader.context().id();
			let id = id.clone().with_relative_basis(reader.context().id(), true);
			Ok(Some(id))
		}
		None => Ok(None),
	}
}

pub fn query_source_req<'doc>(reader: &NodeReader<'doc, NodeContext>) -> anyhow::Result<SourceId> {
	Ok(query_source_opt(reader)?.ok_or(MissingSource)?)
}
