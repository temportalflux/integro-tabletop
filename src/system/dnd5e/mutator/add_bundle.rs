use crate::{
	database::app::Criteria,
	kdl_ext::{AsKdl, FromKDL, KDLNode, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::{
			data::{
				character::{AdditionalBundleData, Character},
				description, Bundle,
			},
			Value,
		},
	},
	utility::{selector, Mutator},
};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct AddBundle {
	category: String,
	selector: selector::Object<Character>,
	propogate_parent: bool,
}

crate::impl_trait_eq!(AddBundle);
crate::impl_kdl_node!(AddBundle, "add_bundle");

impl Mutator for AddBundle {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		description::Section {
			content: "You have the following bundles:".to_owned().into(),
			children: vec![selector::DataList::default()
				.with_object("Bundle", &self.selector, state)
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
		stats.add_bundles(AdditionalBundleData {
			ids: ids.collect(),
			source: parent.to_owned(),
			propagate_source_as_parent_feature: self.propogate_parent,
		});
	}
}

impl FromKDL for AddBundle {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let category = node.get_str_req("category")?.to_owned();
		let amount = node
			.query_opt_t::<Value<i32>>("scope() > amount")?
			.unwrap_or(Value::Fixed(1));
		let selector = selector::Object {
			id: Default::default(),
			object_category: Bundle::id().into(),
			amount,
			criteria: Some(Criteria::contains_prop(
				"category",
				Criteria::exact(category.to_owned()),
			)),
		};
		let propogate_parent = node.get_bool_opt("propogate_parent")?.unwrap_or_default();
		Ok(Self {
			category,
			selector,
			propogate_parent,
		})
	}
}

impl AsKdl for AddBundle {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(("category", self.category.clone()));
		if self.propogate_parent {
			node.push_entry(("propogate_parent", true));
		}
		if self.selector.amount != Value::Fixed(1) {
			node.push_child_t("amount", &self.selector.amount);
		}
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
				selector: selector::Object {
					id: Default::default(),
					object_category: Bundle::id().into(),
					amount: Value::Fixed(1),
					criteria: Some(Criteria::contains_prop(
						"category",
						Criteria::exact("Feat".to_owned()),
					)),
				},
				propogate_parent: false,
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
