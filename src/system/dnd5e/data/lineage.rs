use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt},
	system::dnd5e::{
		data::{character::Character, BoxedFeature},
		BoxedMutator, DnD5e, FromKDL, SystemComponent,
	},
	utility::MutatorGroup,
};

#[derive(Default, Clone, PartialEq)]
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

	fn node_name() -> &'static str {
		"lineage"
	}

	fn add_component(self, system: &mut Self::System) {
		system.add_lineage(self);
	}
}

impl FromKDL for Lineage {
	type System = DnD5e;

	fn from_kdl(node: &kdl::KdlNode, system: &Self::System) -> anyhow::Result<Self> {
		let name = node.get_str("name")?.to_owned();
		let description = node
			.query_str_opt("description", 0)?
			.unwrap_or_default()
			.to_owned();
		let can_select_twice = node.get_bool_opt("can_select_twice")?.unwrap_or_default();
		let mut mutators = Vec::new();
		let mut features = Vec::new();
		if let Some(children) = node.children() {
			for mutator_node in children.query_all("mutator")? {
				let factory = system.get_mutator_factory(mutator_node.get_str(0)?)?;
				mutators.push(factory.from_kdl(mutator_node, system)?);
			}
			for feature_node in children.query_all("feature")? {}
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
