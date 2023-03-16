use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::{
		core::{NodeRegistry, SourceId},
		dnd5e::{
			data::{character::Character, BoxedFeature, Feature},
			BoxedMutator, DnD5e, FromKDL, SystemComponent,
		},
	},
	utility::MutatorGroup,
};
use std::path::Path;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Upbringing {
	pub source_id: SourceId,
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<BoxedFeature>,
}

impl MutatorGroup for Upbringing {
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

crate::impl_kdl_node!(Upbringing, "upbringing");

impl SystemComponent for Upbringing {
	type System = DnD5e;

	fn add_component(mut self, source_id: SourceId, system: &mut Self::System) {
		self.source_id = source_id.clone();
		system.upbringings.insert(source_id, self);
	}
}

impl FromKDL for Upbringing {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(node_reg.parse_mutator(entry_node)?);
		}

		let mut features = Vec::new();
		for entry_node in node.query_all("scope() > feature")? {
			features
				.push(Feature::from_kdl(entry_node, &mut ValueIdx::default(), node_reg)?.into());
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
