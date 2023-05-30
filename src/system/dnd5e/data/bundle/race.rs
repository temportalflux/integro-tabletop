use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{
			data::{character::Character, Feature},
			BoxedMutator, SystemComponent,
		},
	},
	utility::MutatorGroup,
};
use std::path::Path;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Race {
	pub source_id: SourceId,
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<Feature>,
}

impl MutatorGroup for Race {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			mutator.set_data_path(&path_to_self);
		}
		for feature in &self.features {
			feature.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
		for feat in &self.features {
			stats.add_feature(feat, &path_to_self);
		}
	}
}

crate::impl_kdl_node!(Race, "race");

impl SystemComponent for Race {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name.clone(),
		})
	}
}

impl FromKDL for Race {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let mut features = Vec::new();
		for entry_node in node.query_all("scope() > feature")? {
			features.push(Feature::from_kdl(entry_node, &mut ctx.next_node())?.into());
		}

		Ok(Self {
			source_id: SourceId::default(),
			name,
			description,
			mutators,
			features,
		})
	}
}
