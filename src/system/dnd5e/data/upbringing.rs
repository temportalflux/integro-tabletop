use super::Feature;
use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::{
		core::{NodeRegistry, SourceId},
		dnd5e::{
			data::{character::Character, BoxedFeature},
			BoxedMutator, DnD5e, FromKDL, KDLNode, SystemComponent,
		},
	},
	utility::MutatorGroup,
};

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

	fn id(&self) -> Option<String> {
		use convert_case::Casing;
		Some(self.name.to_case(convert_case::Case::Pascal))
	}

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		for mutator in &self.mutators {
			stats.apply(mutator);
		}
		for feat in &self.features {
			stats.add_feature(feat);
		}
	}
}

impl KDLNode for Upbringing {
	fn id() -> &'static str {
		"upbringing"
	}
}

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
			.query_str_opt("description", 0)?
			.unwrap_or_default()
			.to_owned();
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
		Ok(Upbringing {
			source_id: SourceId::default(),
			name,
			description,
			mutators,
			features,
		})
	}
}
