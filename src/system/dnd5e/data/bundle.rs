use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{
			data::{character::Character, description, Ability, Feature},
			BoxedMutator, SystemComponent,
		},
	},
	utility::{MutatorGroup, NotInList},
};
use std::{collections::HashMap, path::Path, str::FromStr};

mod background;
pub use background::*;
mod lineage;
pub use lineage::*;
mod race;
pub use race::*;
mod race_variant;
pub use race_variant::*;
mod upbringing;
pub use upbringing::*;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Bundle {
	pub id: SourceId,
	pub name: String,
	/// The group this bundle is in (Race, RaceVariant, Lineage, Upbringing, Background, Feat, etc).
	pub category: String,
	pub description: description::Section,
	/// The bundles required for this one to be added to a character.
	pub requirements: Vec<BundleRequirement>,
	/// The number of times this bundle can be added to a character.
	pub limit: u32,
	pub mutators: Vec<BoxedMutator>,
	pub features: Vec<Feature>,
}

// TODO: Could bundle requirements just be a criteria/bool-evaluator?
#[derive(Clone, PartialEq, Debug)]
pub enum BundleRequirement {
	/// The character must have a bundle with the specified category and name.
	Bundle { category: String, name: String },
	/// The character must have an ability score of at least a specific amount.
	Ability(Ability, u32),
}

impl MutatorGroup for Bundle {
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

crate::impl_kdl_node!(Bundle, "bundle");

impl SystemComponent for Bundle {
	fn to_metadata(self) -> serde_json::Value {
		let requirements = {
			let mut requirements = HashMap::new();
			for req in self.requirements {
				match req {
					BundleRequirement::Bundle { category, name } => {
						if !requirements.contains_key("Bundle") {
							requirements.insert("Bundle", HashMap::new());
						}
						let reqs = requirements.get_mut("Bundle").unwrap();
						reqs.insert(category, serde_json::json!(name));
					}
					BundleRequirement::Ability(ability, score) => {
						if !requirements.contains_key("Ability") {
							requirements.insert("Ability", HashMap::new());
						}
						let reqs = requirements.get_mut("Ability").unwrap();
						reqs.insert(ability.long_name().to_owned(), serde_json::json!(score));
					}
				}
			}
			requirements
		};
		serde_json::json!({
			"name": self.name,
			"category": self.category,
			"requirements": requirements,
			"limit": self.limit,
		})
	}
}

impl FromKDL for Bundle {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let id = ctx.parse_source_req(node)?;

		let name = node.get_str_req("name")?.to_owned();
		let category = node.get_str_req("category")?.to_owned();
		let description = match node.query_opt("scope() > description")? {
			Some(node) => description::Section::from_kdl(node, &mut ctx.next_node())?,
			None => description::Section::default(),
		};
		let limit = node.get_i64_opt("limit")?.unwrap_or(1) as u32;

		let mut requirements = Vec::new();
		for node in node.query_all("scope() > requirement")? {
			let mut ctx = ctx.next_node();
			match node.get_str_req(ctx.consume_idx())? {
				"Bundle" => {
					let category = node.get_str_req(ctx.consume_idx())?.to_owned();
					let name = node.get_str_req(ctx.consume_idx())?.to_owned();
					requirements.push(BundleRequirement::Bundle { category, name });
				}
				"Ability" => {
					let ability = Ability::from_str(node.get_str_req(ctx.consume_idx())?)?;
					let score = node.get_i64_req(ctx.consume_idx())? as u32;
					requirements.push(BundleRequirement::Ability(ability, score));
				}
				kind => return Err(NotInList(kind.into(), vec!["Bundle", "Ability"]).into()),
			}
		}

		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let mut features = Vec::new();
		for entry_node in node.query_all("scope() > feature")? {
			features.push(Feature::from_kdl(entry_node, &mut ctx.next_node())?.into());
		}

		Ok(Self {
			id,
			name,
			category,
			description,
			requirements,
			limit,
			mutators,
			features,
		})
	}
}
