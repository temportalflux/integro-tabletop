use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder},
	system::dnd5e::data::{character::Character, description, Feature},
	utility::{Mutator, MutatorGroup},
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddFeature(pub Feature);

crate::impl_trait_eq!(AddFeature);
crate::impl_kdl_node!(AddFeature, "feature");

impl Mutator for AddFeature {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		// TODO: Description of mutator is the feature itself
		description::Section {
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.0.set_data_path(parent);
	}

	fn on_insert(&self, stats: &mut Character, parent: &std::path::Path) {
		stats.add_feature(&self.0, parent);
	}
}

impl FromKDL for AddFeature {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		Ok(Self(Feature::from_kdl(node, ctx)?))
	}
}
// TODO AsKdl: tests for AddFeature
impl AsKdl for AddFeature {
	fn as_kdl(&self) -> NodeBuilder {
		self.0.as_kdl()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::{core::NodeRegistry, dnd5e::BoxedMutator};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			NodeRegistry::defaultmut_parse_kdl::<AddFeature>(doc)
		}

		#[test]
		fn base_only() -> anyhow::Result<()> {
			let doc = "mutator \"feature\" name=\"Mutator Feature\"";
			let expected = AddFeature(Feature {
				name: "Mutator Feature".into(),
				..Default::default()
			});
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}
}
