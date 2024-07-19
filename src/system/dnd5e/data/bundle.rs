use super::Feature;
use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::{
			data::{character::Character, description},
			BoxedCriteria, BoxedMutator,
		},
		mutator::{self, ReferencePath},
		Block, SourceId,
	},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder, OmitIfEmpty};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Bundle {
	pub id: SourceId,
	pub name: String,
	/// The group this bundle is in (Race, RaceVariant, Lineage, Upbringing, Background, Feat, etc).
	pub category: String,
	pub description: description::Info,
	// Required conditions for this bundle to be applicable for a given character
	pub requirements: Vec<BoxedCriteria>,
	/// The number of times this bundle can be added to a character.
	pub limit: usize,
	pub mutators: Vec<BoxedMutator>,
	pub feature_config: Option<FeatureConfig>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct FeatureConfig {
	pub parent_path: Option<ReferencePath>,
}

impl mutator::Group for Bundle {
	type Target = Character;

	fn set_data_path(&self, parent: &ReferencePath) {
		let path_to_self = parent.join(&self.name, None);
		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &ReferencePath) {
		for requirement in &self.requirements {
			if let Err(_err) = requirement.evaluate(stats) {
				return;
			}
		}

		if let Some(config) = &self.feature_config {
			let feature = Feature {
				name: self.name.clone(),
				description: self.description.clone(),
				mutators: self.mutators.clone(),
				parent: config.parent_path.as_ref().map(|path| path.display.clone()),
				..Default::default()
			};
			stats.add_feature(feature, parent);
		} else {
			let path_to_self = parent.join(&self.name, None);
			for mutator in &self.mutators {
				stats.apply(mutator, &path_to_self);
			}
		}
	}
}

kdlize::impl_kdl_node!(Bundle, "bundle");

impl Block for Bundle {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name,
			"category": self.category,
			"limit": self.limit,
		})
	}
}

impl FromKdl<NodeContext> for Bundle {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let category = node.get_str_req("category")?.to_owned();

		let id = match category.as_str() {
			"Feat" => crate::kdl_ext::query_source_opt(node)?.unwrap_or_default(),
			_ => crate::kdl_ext::query_source_req(node)?,
		};

		let feature_config = match node.get_bool_opt("display_as_feature")? {
			Some(true) => Some(FeatureConfig::default()),
			_ => None,
		};

		let description = node.query_opt_t::<description::Info>("scope() > description")?.unwrap_or_default();
		let limit = node.get_i64_opt("limit")?.unwrap_or(1) as usize;

		let requirements = node.query_all_t("scope() > requirement")?;
		let mutators = node.query_all_t("scope() > mutator")?;

		Ok(Self { id, name, category, description, requirements, limit, mutators, feature_config })
	}
}
// TODO AsKdl: from/as tests for Bundle
impl AsKdl for Bundle {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.entry(("category", self.category.clone()));
		node.entry(("name", self.name.clone()));
		if let Some(_config) = &self.feature_config {
			node.entry(("display_as_feature", true));
		}

		node.child(("source", &self.id, OmitIfEmpty));

		node.children(("requirement", self.requirements.iter()));

		if self.description != description::Info::default() {
			node.child(("description", &self.description));
		}
		if self.limit > 1 {
			node.entry(("limit", self.limit as i64));
		}

		node.children(("mutator", self.mutators.iter()));

		node
	}
}
