use crate::{
	system::dnd5e::data::description,
	utility::{AsTraitEq, Dependencies, TraitEq},
};
use kdlize::{AsKdl, NodeId};
use std::{path::Path, sync::Arc};

mod factory;
pub use factory::*;
mod generic;
pub use generic::*;

pub type ArcMutator<T> = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;

pub trait Mutator: std::fmt::Debug + TraitEq + AsTraitEq<dyn TraitEq> + NodeId + AsKdl {
	type Target;

	fn dependencies(&self) -> Dependencies {
		Dependencies::default()
	}

	fn set_data_path(&self, _parent: &Path) {}

	fn description(&self, _state: Option<&Self::Target>) -> description::Section {
		description::Section::default()
	}

	fn on_insert(&self, _: &mut Self::Target, _parent: &std::path::Path) {}

	fn apply(&self, _: &mut Self::Target, _parent: &std::path::Path) {}
}

pub trait Group {
	type Target;

	fn set_data_path(&self, parent: &std::path::Path);

	fn apply_mutators(&self, target: &mut Self::Target, parent: &std::path::Path);
}
