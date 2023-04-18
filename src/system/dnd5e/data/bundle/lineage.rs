use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeContext, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{
			data::{character::Character, Feature},
			BoxedMutator, DnD5e, SystemComponent,
		},
	},
	utility::MutatorGroup,
};
use std::path::Path;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Lineage {
	pub source_id: SourceId,
	pub name: String,
	pub description: String,
	pub limit: u32,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<Feature>,
}

impl MutatorGroup for Lineage {
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

	fn apply_mutators(&self, stats: &mut Self::Target, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
		for feat in &self.features {
			stats.add_feature(feat, &path_to_self);
		}
	}
}

impl SystemComponent for Lineage {
	type System = DnD5e;

	fn add_component(mut self, source_id: SourceId, system: &mut Self::System) {
		self.source_id = source_id.clone();
		system.lineages.insert(source_id, self);
	}
}

crate::impl_kdl_node!(Lineage, "lineage");

impl FromKDL for Lineage {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let limit = node.get_i64_opt("limit")?.unwrap_or(1) as u32;

		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let mut features = Vec::new();
		for entry_node in node.query_all("scope() > feature")? {
			features.push(Feature::from_kdl(entry_node, &mut ctx.next_node())?.into());
		}

		Ok(Lineage {
			source_id: SourceId::default(),
			name,
			description,
			limit,
			mutators,
			features,
		})
	}
}
