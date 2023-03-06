use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
	system::dnd5e::{
		data::{character::Character, BoxedFeature},
		BoxedMutator, DnD5e, FromKDL, KDLNode, SystemComponent,
	},
	utility::MutatorGroup,
};

use super::Feature;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Lineage {
	pub name: String,
	pub description: String,
	pub can_select_twice: bool,
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

	fn add_component(self, system: &mut Self::System) {
		system.add_lineage(self);
	}
}

impl KDLNode for Lineage {
	fn id() -> &'static str {
		"lineage"
	}
}

impl FromKDL<DnD5e> for Lineage {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.unwrap_or_default()
			.to_owned();
		let can_select_twice = node.get_bool_opt("can_select_twice")?.unwrap_or_default();
		let mut mutators = Vec::new();
		let mut features = Vec::new();
		if let Some(children) = node.children() {
			for entry_node in children.query_all("mutator")? {
				let mut value_idx = ValueIdx::default();
				let id = entry_node.get_str(value_idx.next())?;
				let factory = system.get_mutator_factory(id)?;
				mutators.push(factory.from_kdl(entry_node, &mut value_idx, system)?);
			}
			for entry_node in children.query_all("feature")? {
				features
					.push(Feature::from_kdl(entry_node, &mut ValueIdx::default(), system)?.into());
			}
		}
		Ok(Lineage {
			name,
			description,
			can_select_twice,
			mutators,
			features,
		})
	}
}
