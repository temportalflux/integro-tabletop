use super::character::Character;
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	system::{
		core::SourceId,
		dnd5e::{BoxedMutator, SystemComponent},
	},
	utility::MutatorGroup,
};
use std::{path::Path, sync::Arc};

mod indirect;
pub use indirect::*;

/// A state a character may be subject to until it is removed.
/// Some conditions are automatically cleared on the next long rest, and any conditions may be manually cleared.
/// Conditions contain a set of mutators and an optional criteria that, if met, applies those mutators.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Condition {
	pub id: Option<SourceId>,
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
}

crate::impl_kdl_node!(Condition, "condition");

impl MutatorGroup for Condition {
	type Target = Character;

	fn set_data_path(&self, parent: &Path) {
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

impl SystemComponent for Condition {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name.clone(),
		})
	}
}

impl FromKDL for Condition {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.query_source_opt()?;

		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let mutators = node.query_all_t("scope() > mutator")?;

		Ok(Self {
			id,
			name,
			description,
			mutators,
		})
	}
}

impl AsKdl for Condition {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("name", self.name.clone()));

		if let Some(id) = &self.id {
			node.push_child_opt_t("source", id);
		}
		node.push_child_opt_t("description", &self.description);

		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
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
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				core::NodeRegistry,
				dnd5e::{data::bounded::BoundValue, evaluator::HasArmorEquipped, mutator::Speed},
			},
		};

		static NODE_NAME: &str = "condition";

		fn node_ctx() -> NodeContext {
			NodeContext::registry({
				let mut reg = NodeRegistry::default();
				reg.register_mutator::<Speed>();
				reg.register_evaluator::<HasArmorEquipped>();
				reg
			})
		}

		#[test]
		fn basic() -> anyhow::Result<()> {
			let doc = "
				|condition name=\"Expedient\" {
				|    description \"You are particularly quick.\"
				|}
			";
			let data = Condition {
				name: "Expedient".into(),
				description: "You are particularly quick.".into(),
				..Default::default()
			};
			assert_eq_fromkdl!(Condition, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn mutators() -> anyhow::Result<()> {
			let doc = "
				|condition name=\"Expedient\" {
				|    description \"You are particularly quick.\"
				|    mutator \"speed\" \"Walking\" (Additive)15
				|}
			";
			let data = Condition {
				name: "Expedient".into(),
				description: "You are particularly quick.".into(),
				mutators: vec![Speed {
					name: "Walking".into(),
					argument: BoundValue::Additive(15),
				}
				.into()],
				..Default::default()
			};
			assert_eq_fromkdl!(Condition, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
