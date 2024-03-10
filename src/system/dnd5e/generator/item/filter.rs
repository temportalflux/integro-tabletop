use crate::{
	kdl_ext::NodeContext,
	system::dnd5e::data::{item::armor, Rarity},
};
use kdlize::{
	ext::{DocumentExt, ValueExt},
	AsKdl, FromKdl, NodeBuilder,
};
use std::{collections::BTreeSet, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct Filter {
	// Tags which must exist on the item for it to be a relevant base.
	pub tags: Vec<String>,
	// If provided, the item must be an equipment that provides armor and be one of the provided types.
	// Providing an empty set implies that all armor types are relevant.
	pub armor: Option<BTreeSet<armor::Kind>>,
	// If provided, the item must have a rarity in the set to be a relevant base.
	// If one of the entries in the set is `None`, then items which have no rarity are relevant.
	// If an empty set is provided, then all item rarities (including no rarity) are relevant.
	pub rarity: BTreeSet<Option<Rarity>>,
}

impl Filter {
	pub fn as_criteria(&self) -> crate::database::Criteria {
		use crate::database::Criteria;
		let mut criteria = Vec::new();

		if !self.tags.is_empty() {
			// What this means:
			// There exists a root-level metadata property named `tags`.
			// That `tags` property is an array, which contains every value contained in the `self.tags` list.
			let tag_matches = self.tags.iter().map(|tag| Criteria::exact(tag.as_str()));
			let contains_match = tag_matches.map(|matcher| Criteria::contains_element(matcher));
			let all_tags_match = Criteria::all(contains_match);
			criteria.push(Criteria::contains_prop("tags", all_tags_match).into());
		}

		if let Some(armor_kinds) = &self.armor {
			// What this means:
			// There exists a root-level metadata property named `equipment`.
			// That `equipment` property is an object that contains a property named `armor`.
			// `armor` is an array, which contains any value that exactly matches the string representing an armor kinds provided.
			let exact_matches = armor_kinds.iter().map(|kind| Criteria::exact(kind.to_string()));
			let contains_match = exact_matches.map(|matcher| Criteria::contains_element(matcher));
			let has_any_kind = Criteria::any(contains_match);
			let has_armor = Criteria::contains_prop("armor", has_any_kind);
			criteria.push(Criteria::contains_prop("equipment", has_armor))
		}

		if !self.rarity.is_empty() {
			// What this means:
			// There exists a root-level metadata property named `rarity`.
			// If `None` is in the list is valid rarities, then the criteria passes if an item does not have the `rarity` root property.
			// For each non-none rarity, we check if the property exists and exactly matches the rarity's string representation.
			let rarity_matches = self.rarity.iter().map(|opt| match opt {
				None => Criteria::missing_prop("rarity"),
				Some(rarity) => Criteria::contains_prop("rarity", Criteria::exact(rarity.to_string())),
			});
			criteria.push(Criteria::any(rarity_matches));
		}

		Criteria::All(criteria)
	}
}

impl FromKdl<NodeContext> for Filter {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let tags = node.query_str_all("scope() > tag", 0)?;
		let tags = tags.into_iter().map(str::to_owned).collect();
		let armor = match node.query_opt("scope() > armor")? {
			None => None,
			Some(node) => {
				let mut kinds = BTreeSet::new();
				for entry in node.entries() {
					match entry.as_str_req()? {
						"Any" => {}
						value => {
							kinds.insert(armor::Kind::from_str(value)?);
						}
					}
				}
				Some(kinds)
			}
		};
		let rarity = match node.query_opt("scope() > rarity")? {
			None => BTreeSet::new(),
			Some(node) => {
				let mut rarities = BTreeSet::new();
				for entry in node.entries() {
					rarities.insert(match entry.as_str_req()? {
						"None" => None,
						value => Some(Rarity::from_str(value)?),
					});
				}
				rarities
			}
		};
		Ok(Self { tags, armor, rarity })
	}
}

impl AsKdl for Filter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		for tag in &self.tags {
			node.push_child_t(("tag", tag));
		}

		if let Some(armor_kinds) = &self.armor {
			let mut node_armor = NodeBuilder::default();
			if armor_kinds.is_empty() {
				node_armor.push_entry("Any");
			} else {
				for kind in armor_kinds {
					node_armor.push_entry(kind.to_string());
				}
			}
			node.push_child(node_armor.build("armor"));
		}

		if !self.rarity.is_empty() {
			let mut node_rarity = NodeBuilder::default();
			for rarity in &self.rarity {
				node_rarity.push_entry(match rarity {
					None => "None".to_owned(),
					Some(rarity) => rarity.to_string(),
				});
			}
			node.push_child(node_rarity.build("rarity"));
		}

		node
	}
}
