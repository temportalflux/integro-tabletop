use crate::{
	system::{generics, SourceId, System},
	GeneralError,
};
use std::sync::Arc;

pub struct Entry(Arc<dyn System + 'static + Send + Sync>);

impl Entry {
	pub(super) fn new<T>(system: T) -> Self
	where
		T: System + 'static + Send + Sync,
	{
		Self(Arc::new(system))
	}

	pub fn id(&self) -> &'static str {
		self.0.get_id()
	}

	pub fn node(&self) -> Arc<generics::Registry> {
		self.0.generics().clone()
	}

	pub fn parse_metadata(&self, node: &kdl::KdlNode, source_id: &SourceId) -> anyhow::Result<serde_json::Value> {
		use anyhow::Context;
		let category = node.name().value().to_owned();
		let Some(comp_factory) = self.0.blocks().get_factory(&category).cloned() else {
			return Err(GeneralError(format!("No component registered with id {category:?}")).into());
		};
		let ctx = crate::kdl_ext::NodeContext::new(Arc::new(source_id.clone()), self.0.generics().clone());
		let metadata = comp_factory
			.metadata_from_kdl(crate::kdl_ext::NodeReader::new_root(node, ctx))
			.with_context(|| format!("Failed to parse {:?}", source_id.to_string()))?;
		match metadata {
			serde_json::Value::Null => Ok(serde_json::Value::Null),
			serde_json::Value::Object(mut metadata) => {
				if let Some(module_id) = &source_id.module {
					metadata.insert("module".into(), serde_json::json!(module_id.to_string()));
				}
				Ok(serde_json::Value::Object(metadata))
			}
			other => {
				return Err(GeneralError(format!(
					"Metadata must be a map, but {} returned {:?}.",
					source_id.to_string(),
					other
				))
				.into());
			}
		}
	}
}
