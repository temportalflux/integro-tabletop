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

	fn description(&self, state: Option<&Character>) -> description::Section {
		let mut section = description::Section {
			title: Some(self.0.name.clone()),
			format_args: self.0.description.format_args.clone(),
			children: self.0.description.sections.clone(),
			..Default::default()
		};
		for mutator in &self.0.mutators {
			section.children.push(mutator.description(state));
		}
		section
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

impl AsKdl for AddFeature {
	fn as_kdl(&self) -> NodeBuilder {
		self.0.as_kdl()
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(AddFeature);

		#[test]
		fn base_only() -> anyhow::Result<()> {
			let doc = "mutator \"feature\" name=\"Mutator Feature\"";
			let data = AddFeature(Feature {
				name: "Mutator Feature".into(),
				..Default::default()
			});
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
