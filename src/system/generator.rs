use super::VariantId;
use crate::{
	system::SourceId,
	utility::{AsTraitEq, PinFutureLifetimeNoSend, TraitEq},
};
use kdlize::{AsKdl, NodeId};
use std::{fmt::Debug, sync::Arc};

mod factory;
pub use factory::*;
mod generic;
pub use generic::*;
mod queue;
pub use queue::*;
mod variant_cache;
pub use variant_cache::*;

pub struct SystemObjectList {
	generator_module: super::ModuleId,
	generator_short_id: String,
	node_registry: Arc<super::generics::Registry>,
	generators: Vec<Generic>,
	variants: Vec<crate::database::Entry>,
}

impl SystemObjectList {
	pub fn new(generator: &impl Generator, node_registry: Arc<super::generics::Registry>) -> Self {
		Self {
			generator_module: generator.source_id().module.clone().unwrap(),
			generator_short_id: generator.short_id().clone(),
			node_registry,
			generators: Vec::new(),
			variants: Vec::new(),
		}
	}

	pub fn variant_id(&self, variant: impl Into<String>) -> VariantId {
		VariantId {
			module: self.generator_module.clone(),
			generator: self.generator_short_id.clone(),
			variant: variant.into(),
		}
	}

	pub fn insert(&mut self, variant_name: impl Into<String>, mut entry: crate::database::Entry) {
		entry.generated = 1;

		if entry.category == Generic::id() {
			let generator = entry.parse_kdl::<Generic>(self.node_registry.clone());
			let generator = generator.expect("Entry had generator id, but failed to parse to generator");
			self.generators.push(generator);
		}

		entry.id = {
			let mut source_id = entry.source_id(true);
			source_id = source_id.into_unversioned();
			source_id.variant = Some(self.variant_id(variant_name));
			source_id.to_string()
		};

		self.variants.push(entry);
	}

	pub fn drain_generators<'this>(&'this mut self) -> impl Iterator<Item = Generic> + 'this {
		self.generators.drain(..)
	}

	pub fn drain_variants<'this>(&'this mut self) -> impl Iterator<Item = crate::database::Entry> + 'this {
		self.variants.drain(..)
	}
}

pub struct Args<'exec> {
	pub system_registry: &'exec crate::system::Registry,
	pub system: &'exec crate::system::registry::Entry,
	pub database: &'exec crate::database::Database,
}

pub trait Generator: Debug + TraitEq + AsTraitEq<dyn TraitEq> + NodeId + AsKdl {
	fn source_id(&self) -> &SourceId;
	fn short_id(&self) -> &String;
	fn execute<'this>(
		&'this self, args: Args<'this>,
	) -> PinFutureLifetimeNoSend<'this, anyhow::Result<SystemObjectList>>;
}

pub type ArcGenerator = Arc<dyn Generator + 'static + Send + Sync>;
