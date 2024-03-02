use super::weapon;
use crate::kdl_ext::NodeContext;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};
use std::str::FromStr;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Restriction {
	pub tags: Vec<String>,
	pub weapon: Option<Weapon>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Weapon {
	pub kind: Option<weapon::Kind>,
	pub has_melee: Option<bool>,
}

impl FromKdl<NodeContext> for Restriction {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let tags = node
			.query_str_all("scope() > tag", 0)?
			.into_iter()
			.map(str::to_owned)
			.collect::<Vec<_>>();
		let weapon = node.query_opt_t::<Weapon>("scope() > weapon")?;
		Ok(Self { tags, weapon })
	}
}
impl AsKdl for Restriction {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_children_t(("tag", self.tags.iter()));
		node.push_child_t(("weapon", &self.weapon));
		node
	}
}

impl FromKdl<NodeContext> for Weapon {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = match node.get_str_opt("kind")? {
			None => None,
			Some(str) => Some(weapon::Kind::from_str(str)?),
		};
		let has_melee = node.get_bool_opt("has_melee")?;
		Ok(Self { kind, has_melee })
	}
}
impl AsKdl for Weapon {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(kind) = &self.kind {
			node.push_entry(("kind", kind.to_string()));
		}
		if let Some(has_melee) = &self.has_melee {
			node.push_entry(("has_melee", *has_melee));
		}
		node
	}
}

impl Restriction {
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

		if let Some(weapon) = &self.weapon {
			let mut weapon_requirements = Vec::new();
			if let Some(kind) = &weapon.kind {
				weapon_requirements.push(Criteria::contains_prop("kind", Criteria::exact(kind.to_string())));
			}
			if let Some(has_melee) = &weapon.has_melee {
				weapon_requirements.push(Criteria::contains_prop("has_range", Criteria::exact(!*has_melee)));
			}
			let weapon_criteria = Criteria::all(weapon_requirements);
			let equipment_criteria = Criteria::contains_prop("weapon", weapon_criteria);
			criteria.push(Criteria::contains_prop("equipment", equipment_criteria));
		}

		Criteria::All(criteria)
	}
}
