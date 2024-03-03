use crate::kdl_ext::NodeContext;
use kdlize::{AsKdl, FromKdl, NodeId};

mod factory;
pub use factory::*;
mod registry;
pub use registry::*;

/// A block (root-level kdl node) which exposes functionality for
/// constructing metadata about the struct, for embedding in the database record.
pub trait Block: FromKdl<NodeContext> + NodeId + AsKdl {
	fn to_metadata(self) -> serde_json::Value
	where
		Self: Sized;
}
