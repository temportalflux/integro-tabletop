use super::{AsTraitEq, Dependencies, TraitEq};
use crate::{
	kdl_ext::{AsKdl, KDLNode},
	system::{dnd5e::data::description, core::SourceId},
};
use std::{fmt::Debug, path::Path, sync::Arc};

pub trait Mutator: Debug + TraitEq + AsTraitEq<dyn TraitEq> + KDLNode + AsKdl {
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

pub type ArcMutator<T> = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;
#[derive(Clone)]
pub struct GenericMutator<T>(ArcMutator<T>);

impl<M, T> From<M> for GenericMutator<T>
where
	M: Mutator<Target = T> + 'static + Send + Sync,
{
	fn from(value: M) -> Self {
		Self(Arc::new(value))
	}
}

impl<T> GenericMutator<T> {
	pub fn new(value: ArcMutator<T>) -> Self {
		Self(value)
	}

	pub fn into_inner(self) -> ArcMutator<T> {
		self.0
	}
}

impl<T> PartialEq for GenericMutator<T>
where
	T: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.0.equals_trait((*other.0).as_trait_eq())
	}
}

impl<T> std::ops::Deref for GenericMutator<T> {
	type Target = Arc<dyn Mutator<Target = T> + 'static + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<T> std::ops::DerefMut for GenericMutator<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.0
    }
}

impl<T> std::fmt::Debug for GenericMutator<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<T> AsKdl for GenericMutator<T> {
	fn as_kdl(&self) -> crate::kdl_ext::NodeBuilder {
		crate::kdl_ext::NodeBuilder::default()
			.with_entry(self.0.get_id())
			.with_extension(self.0.as_kdl())
	}
}

pub trait MutatorGroup {
	type Target;

	fn set_data_path(&self, parent: &Path);

	fn apply_mutators(&self, target: &mut Self::Target, parent: &Path);
}

impl<T> crate::kdl_ext::FromKDL for GenericMutator<T>
where
	T: 'static,
{
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.next_str_req()?;
		let node_reg = node.node_reg().clone();
		let factory = node_reg.get_mutator_factory(id)?;
		factory.from_kdl::<T>(node)
	}
}
