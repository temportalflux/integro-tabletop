use std::collections::BTreeMap;

use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::data::character::Character,
	utility::{Dependencies, GenericMutator, Mutator},
};

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

	fn dependencies(&self) -> Dependencies {
		let mut deps = Dependencies::default();
		for (_level, batch) in &self.levels {
			for mutator in batch {
				deps = deps.join(mutator.dependencies());
			}
		}
		deps
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		for (_level, batch) in &self.levels {
			for mutator in batch {
				mutator.set_data_path(parent);
			}
		}
	}

	fn name(&self) -> Option<String> {
		Some("Grant by Level".into())
	}

	fn description(&self) -> Option<String> {
		let mut desc = format!(
			"You are granted benefits based on your {} level:",
			self.class_name.as_ref().map(String::as_str).unwrap_or("Character")
		);
		for (level, batch) in &self.levels {
			if batch.is_empty() {
				continue;
			}
			let mut rows = Vec::new();
			for mutator in batch {
				let Some(desc) = mutator.description() else { continue; };
				let name = match mutator.name() {
					Some(name) => format!("{name}. "),
					None => String::new(),
				};
				rows.push(format!("    {name}{desc}",));
			}
			desc += &format!("\n  At level {level}:{}", rows.join("\n"));
		}
		Some(desc)
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
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
