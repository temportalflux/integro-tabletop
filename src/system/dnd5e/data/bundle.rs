use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{
			data::{character::Character, description, Ability},
			BoxedMutator, SystemComponent,
		},
	},
	utility::{MutatorGroup, NotInList},
};
use std::{collections::HashMap, path::Path, str::FromStr};

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
	pub limit: usize,
	pub mutators: Vec<BoxedMutator>,
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
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let path_to_self = parent.join(&self.name);
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
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
		let name = node.get_str_req("name")?.to_owned();
		let category = node.get_str_req("category")?.to_owned();

		let id = match category.as_str() {
			"Feat" => ctx.parse_source_opt(node)?.unwrap_or_default(),
			_ => ctx.parse_source_req(node)?,
		};

		let description = match node.query_opt("scope() > description")? {
			Some(node) => description::Section::from_kdl(node, &mut ctx.next_node())?,
			None => description::Section::default(),
		};
		let limit = node.get_i64_opt("limit")?.unwrap_or(1) as usize;

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

		Ok(Self {
			id,
			name,
			category,
			description,
			requirements,
			limit,
			mutators,
		})
	}
}
// TODO AsKdl: from/as tests for Bundle
impl AsKdl for Bundle {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("category", self.category.clone()));
		node.push_entry(("name", self.name.clone()));

		node.push_child_t("source", &self.id);

		for requirement in &self.requirements {
			let kdl = match requirement {
				BundleRequirement::Bundle { category, name } => NodeBuilder::default()
					.with_entry("Bundle")
					.with_entry(category.clone())
					.with_entry(name.clone()),
				BundleRequirement::Ability(ability, score) => NodeBuilder::default()
					.with_entry("Ability")
					.with_entry(ability.long_name())
					.with_entry(*score as i64),
			};
			node.push_child(kdl.build("requirement"));
		}

		if self.description != description::Section::default() {
			node.push_child_t("description", &self.description);
		}
		if self.limit > 1 {
			node.push_entry(("limit", self.limit as i64));
		}

		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
		}

		node
	}
}
