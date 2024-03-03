use super::Block;
use crate::kdl_ext::NodeReader;

type FnMetadataFromKdl = Box<dyn Fn(NodeReader<'_>) -> anyhow::Result<serde_json::Value> + 'static + Send + Sync>;
type FnReserializeKdl = Box<dyn Fn(NodeReader<'_>) -> anyhow::Result<kdl::KdlNode> + 'static + Send + Sync>;

/// A factory which parses a block (root-level kdl node) into some concrete type, and exposes methods for calling
/// specific functions on that type (converting it to database record metadata, or reserializing into text).
pub struct Factory {
	metadata_from_kdl: FnMetadataFromKdl,
	reserialize_kdl: FnReserializeKdl,
}
impl Factory {
	pub(super) fn new<T>() -> Self
	where
		T: Block + 'static + Send + Sync,
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

	pub fn metadata_from_kdl<'doc>(&self, node: NodeReader<'doc>) -> anyhow::Result<serde_json::Value> {
		(*self.metadata_from_kdl)(node)
	}

	pub fn reserialize_kdl<'doc>(&self, node: NodeReader<'doc>) -> anyhow::Result<kdl::KdlNode> {
		(*self.reserialize_kdl)(node)
	}
}
