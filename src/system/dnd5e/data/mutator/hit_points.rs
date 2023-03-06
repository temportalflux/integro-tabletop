use crate::{
	system::dnd5e::{data::character::Character, DnD5e, FromKDL, KDLNode, Value},
	utility::{Dependencies, Evaluator, Mutator},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddMaxHitPoints {
	pub id: Option<String>,
	pub value: Value<i32>,
}

impl crate::utility::TraitEq for AddMaxHitPoints {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
	}
}

impl KDLNode for AddMaxHitPoints {
	fn id() -> &'static str {
		"add_max_hit_points"
	}
}

impl Mutator for AddMaxHitPoints {
	type Target = Character;

	fn get_node_name(&self) -> &'static str {
		Self::id()
	}

	fn dependencies(&self) -> Dependencies {
		self.value.dependencies()
	}

	fn data_id(&self) -> Option<&str> {
		self.id.as_ref().map(String::as_str)
	}

	fn apply<'c>(&self, stats: &mut Character) {
		let source = stats.source_path();
		let value = self.value.evaluate(stats);
		stats.max_hit_points_mut().push(value, source);
	}
}

impl FromKDL<DnD5e> for AddMaxHitPoints {
	fn from_kdl(
		node: &kdl::KdlNode,
		value_idx: &mut crate::kdl_ext::ValueIdx,
		system: &DnD5e,
	) -> anyhow::Result<Self> {
		let value = Value::from_kdl(node, value_idx, system, |value| {
			value.as_i64().map(|v| v as i32)
		})?;
		Ok(Self { id: None, value })
	}
}

// TODO: Test AddMaxHitPoints
#[cfg(test)]
mod test {
	use super::*;
	use crate::system::dnd5e::BoxedMutator;

	fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
		let mut system = DnD5e::default();
		system.register_mutator::<AddMaxHitPoints>();
		system.register_evaluator::<crate::system::dnd5e::data::evaluator::GetAbilityModifier>();
		system.parse_kdl_mutator(doc)
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
