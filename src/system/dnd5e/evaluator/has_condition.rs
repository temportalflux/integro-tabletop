use std::str::FromStr;

use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt, AsKdl, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::data::{character::Character, Condition},
	},
};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasCondition {
	pub inverted: bool,
	pub filters: Vec<ConditionFilter>,
}

crate::impl_trait_eq!(HasCondition);
crate::impl_kdl_node!(HasCondition, "has_condition");

impl crate::utility::Evaluator for HasCondition {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		let filters_named = self
			.filters
			.iter()
			.map(|filter| filter.name.clone())
			.collect();
		let filter_list_english = crate::utility::list_as_english(filters_named, "or");
		Some(match (self.inverted, filter_list_english) {
			(true, None) => format!("you don't have any conditions"),
			(true, Some(list_str)) => format!("you don't have the {list_str} condition(s)"),
			(false, None) => format!("you have any condition"),
			(false, Some(list_str)) => format!("you have the {list_str} condition"),
		})
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		if self.filters.is_empty() {
			let condition_count = character.persistent().conditions.iter().count();
			return match (self.inverted, condition_count) {
				(true, 0) => Ok(()),
				(true, count) => Err(format!("{count} conditions found.")),
				(false, 0) => Err(format!("No conditions found.")),
				(false, _) => Ok(()),
			};
		}
		let mut found_any_match = false;
		'iter_condition: for condition in character.persistent().conditions.iter() {
			for filter in &self.filters {
				// if the condition doesn't match this filter, then it isn't relevant
				if !filter.matches(condition) {
					continue;
				}
				found_any_match = true;
				break 'iter_condition;
			}
		}
		match (self.inverted, found_any_match) {
			// success when inverted is no filters were matched
			(true, false) => Ok(()),
			(true, true) => Err(format!("Found undesirable condition.")),
			// not inverted, so success is if we found a match
			(false, true) => Ok(()),
			(false, false) => Err(format!("No relevant condition found.")),
		}
	}
}

impl FromKDL for HasCondition {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let inverted = node.get_bool_opt("inverted")?.unwrap_or_default();
		let mut filters = Vec::new();
		for node in node.query_all("scope() > filter")? {
			filters.push(ConditionFilter::from_kdl(node, &mut ctx.next_node())?);
		}
		Ok(Self { inverted, filters })
	}
}
// TODO AsKdl: tests for HasCondition
impl AsKdl for HasCondition {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if self.inverted {
			node.push_entry(("inverted", true));
		}
		for filter in &self.filters {
			node.push_child_t("filter", filter);
		}
		node
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ConditionFilter {
	name: String,
	properties: Vec<ConditionProperty>,
}
#[derive(Clone, PartialEq, Debug)]
pub enum ConditionProperty {
	Id(SourceId),
	Name(String),
}

impl FromKDL for ConditionFilter {
	fn from_kdl(
		node: &kdl::KdlNode,
		_ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let mut properties = Vec::new();
		if let Some(value) = node.query_str_opt("scope() > id", 0)? {
			properties.push(ConditionProperty::Id(SourceId::from_str(value)?));
		}
		if let Some(value) = node.query_str_opt("scope() > name", 0)? {
			properties.push(ConditionProperty::Name(value.to_owned()));
		}
		Ok(Self { name, properties })
	}
}
// TODO AsKdl: tests for ConditionFilter
impl AsKdl for ConditionFilter {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(("name", self.name.clone()));
		for property in &self.properties {
			match property {
				ConditionProperty::Id(id) => {
					node.push_child_t("id", id);
				}
				ConditionProperty::Name(name) => {
					node.push_child_t("name", name);
				}
			}
		}
		node
	}
}

impl ConditionFilter {
	pub fn matches(&self, condition: &Condition) -> bool {
		if self.properties.is_empty() {
			return false;
		}
		for property in &self.properties {
			if !property.matches(condition) {
				return false;
			}
		}
		true
	}
}
impl ConditionProperty {
	pub fn matches(&self, condition: &Condition) -> bool {
		match self {
			Self::Id(id) => condition.id.as_ref() == Some(id),
			Self::Name(name) => &condition.name == name,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::{system::core::NodeRegistry, utility::GenericEvaluator};

		fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, Result<(), String>>> {
			NodeRegistry::defaulteval_parse_kdl::<HasCondition>(doc)
		}

		#[test]
		fn has_any() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_condition\"";
			let expected = HasCondition::default();
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}

		#[test]
		fn has_none() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_condition\" inverted=true";
			let expected = HasCondition {
				inverted: true,
				..Default::default()
			};
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}

		#[test]
		fn has_some_id() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_condition\" {
				filter name=\"ConditionA\" {
					id \"path/to/condition.kdl\"
				}
			}";
			let expected = HasCondition {
				filters: vec![ConditionFilter {
					name: "ConditionA".into(),
					properties: vec![ConditionProperty::Id(SourceId {
						path: "path/to/condition.kdl".into(),
						..Default::default()
					})],
				}],
				..Default::default()
			};
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}

		#[test]
		fn has_some_name() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_condition\" {
				filter name=\"ConditionA\" {
					name \"CustomCondition\"
				}
			}";
			let expected = HasCondition {
				filters: vec![ConditionFilter {
					name: "ConditionA".into(),
					properties: vec![ConditionProperty::Name("CustomCondition".into())],
				}],
				..Default::default()
			};
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}
	}

	// TODO: Test has_condition evaluate
	mod evaluate {
		use super::*;
		use crate::{system::dnd5e::data::character::Persistent, utility::Evaluator};

		fn character(conditions: Vec<Condition>) -> Character {
			let mut persistent = Persistent::default();
			for condition in conditions {
				persistent.conditions.insert(condition);
			}
			Character::from(persistent)
		}

		#[test]
		fn has_any() {
			let evaluator = HasCondition::default();
			let character = character(vec![]);
			assert!(evaluator.evaluate(&character).is_err());
		}
	}
}
