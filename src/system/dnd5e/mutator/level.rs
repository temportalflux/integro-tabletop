use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{character::Character, description},
	utility::{GenericMutator, Mutator},
};
use std::collections::BTreeMap;

// Grants child mutators based on the character's level.
#[derive(Clone, PartialEq, Debug)]
pub struct GrantByLevel {
	class_name: Option<String>,
	levels: BTreeMap<usize, Vec<GenericMutator<Character>>>,
}

crate::impl_trait_eq!(GrantByLevel);
crate::impl_kdl_node!(GrantByLevel, "by_level");

impl Mutator for GrantByLevel {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		for (_level, batch) in &self.levels {
			for mutator in batch {
				mutator.set_data_path(parent);
			}
		}
	}

	fn description(&self, state: Option<&Character>) -> description::Section {
		let mut sections = Vec::new();

		for (level, batch) in &self.levels {
			if batch.is_empty() {
				continue;
			}
			let children: Vec<_> = batch
				.iter()
				.map(|mutator| mutator.description(state))
				.collect();
			if children.is_empty() {
				continue;
			}
			sections.push(description::Section {
				title: Some(format!("Level {level}")),
				children: children,
				..Default::default()
			})
		}

		description::Section {
			title: Some("Grant by Level".into()),
			content: format!(
				"You are granted benefits based on your {} level:",
				self.class_name
					.as_ref()
					.map(String::as_str)
					.unwrap_or("Character")
			)
			.into(),
			children: sections,
			..Default::default()
		}
	}

	// This needs to be run before the cached mutators are applied, otherwise
	// the mutators inserted during this function are never truely applied.
	fn on_insert(&self, stats: &mut Character, parent: &std::path::Path) {
		let current_level = stats.level(self.class_name.as_ref().map(String::as_str));
		for (level, batch) in &self.levels {
			if *level > current_level {
				break;
			}
			for mutator in batch {
				stats.apply(mutator, parent);
			}
		}
	}
}

impl FromKDL for GrantByLevel {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt("class")?.map(str::to_owned);

		let mut levels = BTreeMap::new();
		for node in node.query_all("scope() > level")? {
			let mut ctx = ctx.next_node();

			let level = node.get_i64_req(ctx.consume_idx())? as usize;

			let mut mutators = Vec::new();
			for node in node.query_all("scope() > mutator")? {
				let ctx = ctx.next_node();
				mutators.push(ctx.parse_mutator(node)?);
			}

			levels.insert(level, mutators);
		}

		Ok(Self { class_name, levels })
	}
}
// TODO AsKdl: tests for GrantByLevel
impl AsKdl for GrantByLevel {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(class_name) = &self.class_name {
			node.push_entry(("class", class_name.clone()));
		}
		for (level, mutators) in &self.levels {
			node.push_child({
				let mut node = NodeBuilder::default().with_entry(*level as i64);
				for mutator in mutators {
					node.push_child_t("mutator", mutator);
				}
				node.build("level")
			})
		}
		node
	}
}
