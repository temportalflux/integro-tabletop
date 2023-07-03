use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::{
		data::{character::Character, description},
		BoxedCriteria, BoxedMutator,
	},
	utility::{Mutator, NotInList},
};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ApplyIf {
	op: LogicOp,
	criteria: Vec<BoxedCriteria>,
	mutators: Vec<BoxedMutator>,
}

#[derive(Clone, PartialEq, Debug, Default)]
enum LogicOp {
	#[default]
	All, // and
	Any, // or
}
impl ToString for LogicOp {
	fn to_string(&self) -> String {
		match self {
			Self::All => "All",
			Self::Any => "Any",
		}
		.into()
	}
}
impl FromStr for LogicOp {
	type Err = NotInList;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"All" => Ok(LogicOp::All),
			"Any" => Ok(LogicOp::Any),
			_ => Err(NotInList(s.into(), vec!["All", "Any"])),
		}
	}
}

crate::impl_trait_eq!(ApplyIf);
crate::impl_kdl_node!(ApplyIf, "apply_if");

impl Mutator for ApplyIf {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		let mut criteria_desc = Vec::new();
		for criteria in &self.criteria {
			if let Some(desc) = criteria.description() {
				criteria_desc.push(description::Section { content: desc.into(), ..Default::default() });
			}
		}
		let mut mutator_desc = Vec::new();
		for mutator in &self.mutators {
			mutator_desc.push(mutator.description(state));
		}
		description::Section {
			title: Some("Apply If".into()),
			children: vec![
				description::Section {
					title: Some("Criteria".into()),
					children: criteria_desc,
					..Default::default()
				},
				description::Section {
					title: Some("Applied Changes".into()),
					children: mutator_desc,
					..Default::default()
				},
			],
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		for mutator in &self.mutators {
			mutator.set_data_path(parent);
		}
	}

	fn apply(&self, state: &mut Self::Target, parent: &std::path::Path) {
		if self.evaluate(state) {
			for mutator in &self.mutators {
				state.apply(mutator, parent);
			}
		}
	}
}

impl ApplyIf {
	fn evaluate(&self, state: &Character) -> bool {
		for criteria in &self.criteria {
			// TODO: Somehow save the error text for display in feature UI
			let passed = state.evaluate(criteria).is_ok();
			match self.op {
				LogicOp::All => match passed {
					true => {}             // must pass every criteria
					false => return false, // any criteria failed causes early failure
				},
				LogicOp::Any => match passed {
					true => return true, // any criteria can pass
					false => {}          // every criteria failed causes late failure
				},
			}
		}
		// when loop naturally ends, either:
		// a) op is All and every criteria passed
		// b) op is Any and every criteria failed
		self.op == LogicOp::All
	}
}

impl FromKDL for ApplyIf {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let op = match node.get_str_opt(ctx.consume_idx())? {
			None => LogicOp::default(),
			Some(str) => LogicOp::from_str(str)?,
		};
		let mut criteria = Vec::new();
		for node in node.query_all("scope() > criteria")? {
			criteria.push(ctx.parse_evaluator(node)?);
		}
		let mut mutators = Vec::new();
		for node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(node)?);
		}
		Ok(Self {
			op,
			criteria,
			mutators,
		})
	}
}

impl AsKdl for ApplyIf {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		// allow supressing of the op if-and-only-if we either:
		// - are evaluating only 1 criteria
		// - require all n-criteria
		if self.op != LogicOp::All || self.criteria.len() > 1 {
			node.push_entry(self.op.to_string());
		}
		for criteria in &self.criteria {
			node.push_child({
				let mut node = criteria.as_kdl();
				node.set_first_entry_ty("Evaluator");
				node.build("criteria")
			});
		}
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
			kdl_ext::test_utils::*,
			system::{dnd5e::{
				evaluator::HasArmorEquipped,
				mutator::{test::test_utils, Bonus},
			}, core::NodeRegistry},
		};

		test_utils!(ApplyIf, node_reg());

		fn node_reg() -> NodeRegistry {
			let mut node_reg = NodeRegistry::default();
			node_reg.register_mutator::<ApplyIf>();
			node_reg.register_evaluator::<HasArmorEquipped>();
			node_reg.register_mutator::<Bonus>();
			node_reg
		}

		#[test]
		fn default() -> anyhow::Result<()> {
			let doc = "mutator \"apply_if\"";
			let data = ApplyIf::default();
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn single_criteria() -> anyhow::Result<()> {
			let doc = "
				|mutator \"apply_if\" {
				|    criteria (Evaluator)\"has_armor_equipped\"
				|}
			";
			let data = ApplyIf {
				criteria: vec![HasArmorEquipped::default().into()],
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn all_criteria() -> anyhow::Result<()> {
			let doc = "
				|mutator \"apply_if\" {
				|    criteria (Evaluator)\"has_armor_equipped\"
				|    criteria (Evaluator)\"has_armor_equipped\" inverted=true
				|}
			";
			let data = ApplyIf {
				criteria: vec![
					HasArmorEquipped::default().into(),
					HasArmorEquipped {
						inverted: true,
						..Default::default()
					}
					.into(),
				],
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn any_criteria() -> anyhow::Result<()> {
			let doc = "
				|mutator \"apply_if\" \"Any\" {
				|    criteria (Evaluator)\"has_armor_equipped\"
				|    criteria (Evaluator)\"has_armor_equipped\" inverted=true
				|}
			";
			let data = ApplyIf {
				op: LogicOp::Any,
				criteria: vec![
					HasArmorEquipped::default().into(),
					HasArmorEquipped {
						inverted: true,
						..Default::default()
					}
					.into(),
				],
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn with_mutators() -> anyhow::Result<()> {
			let doc = "
				|mutator \"apply_if\" {
				|    criteria (Evaluator)\"has_armor_equipped\"
				|    mutator \"bonus\" \"ArmorClass\" 2
				|}
			";
			let data = ApplyIf {
				criteria: vec![HasArmorEquipped::default().into()],
				mutators: vec![Bonus::ArmorClass {
					bonus: 2,
					context: None,
				}
				.into()],
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
