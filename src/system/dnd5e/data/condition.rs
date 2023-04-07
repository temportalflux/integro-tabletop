use super::character::Character;
use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{BoxedCriteria, BoxedMutator, DnD5e, SystemComponent},
	},
	utility::MutatorGroup,
};
use std::path::Path;

mod indirect;
pub use indirect::*;

/// A state a character may be subject to until it is removed.
/// Some conditions are automatically cleared on the next long rest, and any conditions may be manually cleared.
/// Conditions contain a set of mutators and an optional criteria that, if met, applies those mutators.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Condition {
	pub source_id: Option<SourceId>,
	pub name: String,
	pub description: String,
	pub mutators: Vec<BoxedMutator>,
	pub criteria: Option<BoxedCriteria>,
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
		if let Some(criteria) = &self.criteria {
			// TODO: Somehow save the error text for display in feature UI
			if stats.evaluate(criteria).is_err() {
				return;
			}
		}
		for mutator in &self.mutators {
			stats.apply(mutator, &path_to_self);
		}
	}
}

impl SystemComponent for Condition {
	type System = DnD5e;

	fn add_component(mut self, source_id: SourceId, system: &mut Self::System) {
		self.source_id = Some(source_id.clone());
		system.conditions.insert(source_id, self);
	}
}

impl FromKDL for Condition {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let description = node
			.query_str_opt("scope() > description", 0)?
			.unwrap_or_default()
			.to_owned();
		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let criteria = match node.query("scope() > criteria")? {
			None => None,
			Some(entry_node) => {
				Some(ctx.parse_evaluator::<Character, Result<(), String>>(entry_node)?)
			}
		};

		Ok(Self {
			source_id: None,
			name,
			description,
			mutators,
			criteria,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;
	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{
				core::NodeRegistry,
				dnd5e::data::{
					bounded::BoundValue, evaluator::armor::HasArmorEquipped, mutator::Speed,
				},
			},
		};

		fn from_doc(doc: &str) -> anyhow::Result<Condition> {
			let mut ctx = NodeContext::registry({
				let mut reg = NodeRegistry::default();
				reg.register_mutator::<Speed>();
				reg.register_evaluator::<HasArmorEquipped>();
				reg
			});
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > condition")?
				.expect("missing condition node");
			Condition::from_kdl(node, &mut ctx)
		}

		#[test]
		fn basic() -> anyhow::Result<()> {
			let doc = "condition name=\"Expedient\" {
			description \"You are particularly quick.\"
		}";
			let expected = Condition {
				name: "Expedient".into(),
				description: "You are particularly quick.".into(),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn mutators() -> anyhow::Result<()> {
			let doc = "condition name=\"Expedient\" {
			description \"You are particularly quick.\"
			mutator \"speed\" \"Walking\" (Additive)15
		}";
			let expected = Condition {
				name: "Expedient".into(),
				description: "You are particularly quick.".into(),
				mutators: vec![Speed {
					name: "Walking".into(),
					argument: BoundValue::Additive(15),
				}
				.into()],
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn criteria() -> anyhow::Result<()> {
			let doc = "condition name=\"Expedient\" {
			description \"You are particularly quick, when not wearing armor.\"
			mutator \"speed\" \"Walking\" (Additive)15
			criteria (Evaluator)\"has_armor_equipped\" inverted=true
		}";
			let expected = Condition {
				name: "Expedient".into(),
				description: "You are particularly quick, when not wearing armor.".into(),
				mutators: vec![Speed {
					name: "Walking".into(),
					argument: BoundValue::Additive(15),
				}
				.into()],
				criteria: Some(
					HasArmorEquipped {
						inverted: true,
						..Default::default()
					}
					.into(),
				),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
