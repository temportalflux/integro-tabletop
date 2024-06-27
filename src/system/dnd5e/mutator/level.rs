use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::data::{character::Character, description},
		mutator::{self, ReferencePath},
		Mutator,
	},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::collections::BTreeMap;

// Grants child mutators based on the character's level.
#[derive(Clone, PartialEq, Debug)]
pub struct GrantByLevel {
	class_name: Option<String>,
	levels: BTreeMap<usize, Vec<mutator::Generic<Character>>>,
}

crate::impl_trait_eq!(GrantByLevel);
kdlize::impl_kdl_node!(GrantByLevel, "by_level");

impl Mutator for GrantByLevel {
	type Target = Character;

	fn set_data_path(&self, parent: &ReferencePath) {
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
			let children: Vec<_> = batch.iter().map(|mutator| mutator.description(state)).collect();
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
				self.class_name.as_ref().map(String::as_str).unwrap_or("Character")
			)
			.into(),
			children: sections,
			..Default::default()
		}
	}

	// This needs to be run before the cached mutators are applied, otherwise
	// the mutators inserted during this function are never truely applied.
	fn on_insert(&self, stats: &mut Character, parent: &ReferencePath) {
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

impl FromKdl<NodeContext> for GrantByLevel {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let class_name = node.get_str_opt("class")?.map(str::to_owned);

		let mut levels = BTreeMap::new();
		for node in &mut node.query_all("scope() > level")? {
			let level = node.next_i64_req()? as usize;
			let mutators = node.query_all_t("scope() > mutator")?;
			levels.insert(level, mutators);
		}

		Ok(Self { class_name, levels })
	}
}

impl AsKdl for GrantByLevel {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(class_name) = &self.class_name {
			node.entry(("class", class_name.clone()));
		}
		for (level, mutators) in &self.levels {
			node.child({
				let mut node = NodeBuilder::default().with_entry(*level as i64);
				node.children(("mutator", mutators.iter()));
				node.build("level")
			})
		}
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::{
				dnd5e::{
					data::character::StatOperation,
					mutator::{test::test_utils, Speed, StatMutator},
				},
				generics,
			},
		};

		test_utils!(GrantByLevel, node_reg());

		fn node_reg() -> generics::Registry {
			let mut node_reg = generics::Registry::default();
			node_reg.register_mutator::<GrantByLevel>();
			node_reg.register_mutator::<Speed>();
			node_reg
		}

		#[test]
		fn empty() -> anyhow::Result<()> {
			let doc = "mutator \"by_level\"";
			let data = GrantByLevel { class_name: None, levels: [].into() };
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn character_level() -> anyhow::Result<()> {
			let doc = "
				|mutator \"by_level\" {
				|    level 3 {
				|        mutator \"speed\" \"Climbing\" (Base)30
				|    }
				|    level 5 {
				|        mutator \"speed\" \"Climbing\" (Add)10
				|    }
				|}
			";
			let data = GrantByLevel {
				class_name: None,
				levels: [
					(3, vec![
						Speed(StatMutator { stat_name: "Climbing".into(), operation: StatOperation::Base(30) }).into(),
					]),
					(5, vec![
						Speed(StatMutator { stat_name: "Climbing".into(), operation: StatOperation::AddSubtract(10) })
							.into(),
					]),
				]
				.into(),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn class_level() -> anyhow::Result<()> {
			let doc = "
				|mutator \"by_level\" class=\"Barbarian\" {
				|    level 3 {
				|        mutator \"speed\" \"Climbing\" (Base)30
				|    }
				|    level 5 {
				|        mutator \"speed\" \"Climbing\" (Add)10
				|    }
				|}
			";
			let data = GrantByLevel {
				class_name: Some("Barbarian".into()),
				levels: [
					(3, vec![
						Speed(StatMutator { stat_name: "Climbing".into(), operation: StatOperation::Base(30) }).into(),
					]),
					(5, vec![
						Speed(StatMutator { stat_name: "Climbing".into(), operation: StatOperation::AddSubtract(10) })
							.into(),
					]),
				]
				.into(),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
