use crate::kdl_ext::NodeContext;
use crate::{
	database::Criteria,
	system::{
		core::SourceId,
		dnd5e::{
			data::{
				character::{AdditionalObjectData, Character},
				description, Bundle,
			},
			Value,
		},
	},
	utility::{selector, Mutator},
	GeneralError,
};
use kdlize::{ext::ValueExt, AsKdl, FromKdl, NodeBuilder};
use kdlize::{NodeId, OmitIfEmpty};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Clone, Debug, PartialEq)]
pub struct AddBundle {
	propogate_parent: bool,
	filter: MetadataObject,
	selector: selector::Object<Character>,
}

#[derive(Clone, Debug, PartialEq)]
struct MetadataObject(BTreeMap<String, MetadataEntry>);
#[derive(Clone, Debug, PartialEq)]
enum MetadataEntry {
	Filter(Vec<String>),
	Object(MetadataObject),
}

crate::impl_trait_eq!(AddBundle);
kdlize::impl_kdl_node!(AddBundle, "add_bundle");

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
		let Some(data_path) = self.selector.get_data_path() else {
			return;
		};
		let Some(selections) = stats.get_selections_at(&data_path) else {
			return;
		};
		let ids = selections.iter().filter_map(|str| SourceId::from_str(str).ok());
		stats.add_bundles(AdditionalObjectData {
			ids: ids.collect(),
			object_type_id: self.selector.object_category.clone(),
			source: parent.to_owned(),
			propagate_source_as_parent_feature: self.propogate_parent,
		});
	}
}

impl FromKdl<NodeContext> for AddBundle {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let object_kind = match node.get_str_opt("object")? {
			None => Bundle::id().into(),
			Some(kind) => kind.to_owned(),
		};
		let propogate_parent = node.get_bool_opt("propogate_parent")?.unwrap_or_default();
		let amount = node
			.query_opt_t::<Value<i32>>("scope() > amount")?
			.unwrap_or(Value::Fixed(1));
		let filter = node.query_req_t::<MetadataObject>("scope() > filter")?;

		let selector = selector::Object {
			id: Default::default(),
			object_category: object_kind,
			amount,
			criteria: Some(filter.as_criteria()),
		};
		Ok(Self {
			selector,
			filter,
			propogate_parent,
		})
	}
}
impl FromKdl<NodeContext> for MetadataObject {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let mut object = Self(BTreeMap::new());
		for node_entry in node.entries() {
			if let Some(id) = node_entry.name() {
				let value = node_entry.as_str_req()?;
				object.insert(id.value(), MetadataEntry::Filter(vec![value.to_owned()]))?;
			}
		}
		if let Some(children) = node.children() {
			for mut node in children {
				let new_entry = MetadataEntry::from_kdl(&mut node)?;
				object.insert(node.name().value(), new_entry)?;
			}
		}
		Ok(object)
	}
}
impl MetadataObject {
	fn insert(&mut self, name: &str, entry: MetadataEntry) -> anyhow::Result<()> {
		if let Some(existing_entry) = self.0.get_mut(name) {
			existing_entry.extend(entry)?;
		} else {
			self.0.insert(name.to_owned(), entry);
		}
		Ok(())
	}
}
impl FromKdl<NodeContext> for MetadataEntry {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		if node.children().is_some() {
			Ok(Self::Object(MetadataObject::from_kdl(node)?))
		} else {
			let mut options = Vec::new();
			for entry in node.entries() {
				if entry.name().is_none() {
					options.push(entry.as_str_req()?.to_owned());
				}
			}
			Ok(Self::Filter(options))
		}
	}
}
impl MetadataEntry {
	fn extend(&mut self, other: MetadataEntry) -> anyhow::Result<()> {
		match (self, other) {
			(Self::Filter(existing), Self::Filter(new)) => {
				existing.extend(new);
				Ok(())
			}
			(Self::Object(existing), Self::Object(new)) => {
				for (key, entry) in new.0 {
					existing.insert(&key, entry)?;
				}
				Ok(())
			}
			(a, b) => Err(GeneralError(format!(
				"Cannot merge an entry of values with an entry object: {:?} & {:?}",
				*a, b
			))
			.into()),
		}
	}
}

impl MetadataObject {
	fn as_criteria(&self) -> Criteria {
		let iter = self.0.iter();
		let iter = iter.map(|(key, entry)| Criteria::contains_prop(key, entry.as_criteria()));
		Criteria::all(iter)
	}
}
impl MetadataEntry {
	fn as_criteria(&self) -> Criteria {
		match self {
			Self::Object(object) => object.as_criteria(),
			Self::Filter(values) => {
				let iter = values.iter();
				let iter = iter.map(|value| Criteria::exact(value.clone()));
				Criteria::any(iter)
			}
		}
	}
}

impl AsKdl for AddBundle {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if self.propogate_parent {
			node.push_entry(("propogate_parent", true));
		}
		if self.selector.object_category != Bundle::id() {
			node.push_entry(("object", self.selector.object_category.as_str()));
		}
		if self.selector.amount != Value::Fixed(1) {
			node.push_child_t(("amount", &self.selector.amount));
		}
		node.push_child_t(("filter", &self.filter, OmitIfEmpty));
		node
	}
}
impl AsKdl for MetadataObject {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		for (key, entry) in &self.0 {
			match entry {
				MetadataEntry::Object(object) => {
					node.push_child_t((key.as_str(), object));
				}
				MetadataEntry::Filter(values) => {
					if values.len() == 1 {
						node.push_entry((key.as_str(), values[0].as_str()));
					} else {
						node.push_children_t((key.as_str(), values.iter()));
					}
				}
			}
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::dnd5e::{data::Subclass, mutator::test::test_utils},
		};

		test_utils!(AddBundle);

		#[test]
		fn feat() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_bundle\" {
				|    filter category=\"Feat\"
				|}
			";
			let data = AddBundle {
				propogate_parent: false,
				filter: MetadataObject([("category".into(), MetadataEntry::Filter(["Feat".into()].into()))].into()),
				selector: selector::Object {
					id: Default::default(),
					object_category: Bundle::id().into(),
					amount: Value::Fixed(1),
					criteria: Some(Criteria::all([Criteria::contains_prop(
						"category",
						Criteria::any([Criteria::exact("Feat".to_owned())]),
					)])),
				},
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn subclass() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_bundle\" object=\"subclass\" {
				|    filter category=\"Cleric\"
				|}
			";
			let data = AddBundle {
				propogate_parent: false,
				filter: MetadataObject([("category".into(), MetadataEntry::Filter(["Cleric".into()].into()))].into()),
				selector: selector::Object {
					id: Default::default(),
					object_category: Subclass::id().into(),
					amount: Value::Fixed(1),
					criteria: Some(Criteria::all([Criteria::contains_prop(
						"category",
						Criteria::any([Criteria::exact("Cleric".to_owned())]),
					)])),
				},
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
