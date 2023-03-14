use super::Feature;
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::{
		core::SourceId,
		dnd5e::{
			data::{character::Character, BoxedFeature},
			BoxedMutator, DnD5e, FromKDL, SystemComponent,
		},
	},
	utility::MutatorGroup,
};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Lineage {
	pub source_id: SourceId,
	pub name: String,
	pub description: String,
	pub limit: u32,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<BoxedFeature>,
}

impl MutatorGroup for Lineage {
	type Target = Character;

	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut Self::Target) {
		for mutator in &self.mutators {
			stats.apply(mutator);
		}
		for feat in &self.features {
			stats.add_feature(feat);
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
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		node_reg: &crate::system::core::NodeRegistry,
	) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.unwrap_or_default()
			.to_owned();
		let limit = node.get_i64_opt("limit")?.unwrap_or(1) as u32;
		let mut mutators = Vec::new();
		let mut features = Vec::new();
		if let Some(children) = node.children() {
			for entry_node in children.query_all("mutator")? {
				mutators.push(node_reg.parse_mutator(entry_node)?);
			}
			for entry_node in children.query_all("feature")? {
				features.push(
					Feature::from_kdl(entry_node, &mut ValueIdx::default(), node_reg)?.into(),
				);
			}
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
