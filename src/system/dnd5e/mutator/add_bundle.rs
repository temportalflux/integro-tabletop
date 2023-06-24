use crate::{
	database::app::Criteria,
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::{
		core::SourceId,
		dnd5e::data::{
			character::Character,
			description,
			Bundle,
		},
	},
	utility::{Mutator, ObjectSelector, SelectorMetaVec},
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct AddBundle {
	category: String,
	selector: ObjectSelector,
}

crate::impl_trait_eq!(AddBundle);
crate::impl_kdl_node!(AddBundle, "add_bundle");

impl Mutator for AddBundle {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		description::Section {
			content: "You have the following bundles:".to_owned().into(),
			children: vec![SelectorMetaVec::default()
				.with_object("Bundle", &self.selector)
				.into()],
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.selector.set_data_path(parent);
	}

	fn on_insert(&self, stats: &mut Character, parent: &std::path::Path) {
		let Some(data_path) = self.selector.get_data_path() else { return; };
		let Some(selections) = stats.get_selections_at(&data_path) else { return; };
		let ids = selections
			.iter()
			.filter_map(|str| SourceId::from_str(str).ok());
		stats.add_bundles(ids.collect(), parent);
	}
}

impl AddBundle {
	fn bundle_selector(category: &str) -> ObjectSelector {
		use crate::kdl_ext::KDLNode;
		let mut selector = ObjectSelector::new(Bundle::id(), 1);
		selector.set_criteria(Criteria::contains_prop(
			"category",
			Criteria::exact(category.to_owned()),
		));
		selector
	}
}

impl FromKDL for AddBundle {
	fn from_kdl(
		node: &kdl::KdlNode,
		_ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let category = node.get_str_req("category")?.to_owned();
		let selector = Self::bundle_selector(&category);
		Ok(Self { category, selector })
	}
}

impl AsKdl for AddBundle {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(("category", self.category.clone()));
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{kdl_ext::test_utils::*, system::dnd5e::mutator::test::test_utils};

		test_utils!(AddBundle);

		#[test]
		fn feat() -> anyhow::Result<()> {
			let doc = "mutator \"add_bundle\" category=\"Feat\"";
			let data = AddBundle {
				category: "Feat".into(),
				selector: AddBundle::bundle_selector("Feat"),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
