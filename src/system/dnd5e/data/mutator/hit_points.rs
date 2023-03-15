use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::character::Character, FromKDL, Value},
	},
	utility::{Dependencies, Evaluator, Mutator},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddMaxHitPoints {
	pub id: Option<String>, // TODO: Unused, should display this in the name of the mutation source
	pub value: Value<i32>,
}

crate::impl_trait_eq!(AddMaxHitPoints);
crate::impl_kdl_node!(AddMaxHitPoints, "add_max_hit_points");

impl Mutator for AddMaxHitPoints {
	type Target = Character;

	fn dependencies(&self) -> Dependencies {
		self.value.dependencies()
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		let value = self.value.evaluate(stats);
		stats.max_hit_points_mut().push(value, parent.to_owned());
	}
}

impl FromKDL for AddMaxHitPoints {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let value = Value::from_kdl(
			node,
			node.entry_req(value_idx.next())?,
			value_idx,
			node_reg,
			|value| Ok(value.as_i64().map(|v| v as i32)),
		)?;
		Ok(Self { id: None, value })
	}
}

// TODO: Test AddMaxHitPoints
#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::BoxedMutator;

	fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
		let mut node_reg = NodeRegistry::default();
		node_reg.register_mutator::<AddMaxHitPoints>();
		node_reg.register_evaluator::<crate::system::dnd5e::data::evaluator::GetAbilityModifier>();
		node_reg.parse_kdl_mutator(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn value() -> anyhow::Result<()> {
			let doc = "mutator \"add_max_hit_points\" 5";
			let expected = AddMaxHitPoints {
				id: None,
				value: Value::Fixed(5),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}

		#[test]
		fn evaluator() -> anyhow::Result<()> {
			let doc = "mutator \"add_max_hit_points\" (Evaluator)\"get_ability_modifier\" \"CON\"";
			let expected = AddMaxHitPoints {
				id: None,
				value: Value::Evaluated(
					crate::system::dnd5e::data::evaluator::GetAbilityModifier(
						crate::system::dnd5e::data::Ability::Constitution,
					)
					.into(),
				),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}
}
