use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt, ValueExt},
	system::dnd5e::{
		data::{character::Character, description},
		Value,
	},
	utility::{Dependencies, Mutator},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AddMaxHitPoints {
	pub id: Option<String>,
	pub value: Value<i32>,
}

crate::impl_trait_eq!(AddMaxHitPoints);
crate::impl_kdl_node!(AddMaxHitPoints, "add_max_hit_points");

impl Mutator for AddMaxHitPoints {
	type Target = Character;

	fn dependencies(&self) -> Dependencies {
		self.value.dependencies()
	}

	fn description(&self, _state: Option<&Character>) -> description::Section {
		static PREFIX: &'static str = "Your hit point maximum increases by";
		description::Section {
			content: match &self.value {
				Value::Fixed(amount) => format!("{PREFIX} {amount}."),
				Value::Evaluated(evaluator) => format!(
					"{PREFIX} {}.",
					match evaluator.description() {
						Some(desc) => desc,
						None => "some amount".into(),
					}
				),
			}
			.into(),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, parent: &std::path::Path) {
		let value = self.value.evaluate(stats);
		let source = match &self.id {
			Some(id) => parent.join(id),
			None => parent.to_owned(),
		};
		stats.max_hit_points_mut().push(value, source);
	}
}

impl FromKDL for AddMaxHitPoints {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let id = node.get_str_opt("id")?.map(str::to_owned);
		let entry = node.next_req()?;
		let value = Value::from_kdl(node, entry, |value| Ok(value.as_i64_req()? as i32))?;
		Ok(Self { id, value })
	}
}

impl AsKdl for AddMaxHitPoints {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(id) = &self.id {
			node.push_entry(("id", id.clone()));
		}
		node += self.value.as_kdl();
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
				core::NodeRegistry,
				dnd5e::{
					data::Ability,
					evaluator::{GetAbilityModifier, GetLevelInt, Math, MathOp},
					mutator::test::test_utils,
				},
			},
		};

		test_utils!(AddMaxHitPoints, node_reg());

		fn node_reg() -> NodeRegistry {
			let mut node_reg = NodeRegistry::default();
			node_reg.register_mutator::<AddMaxHitPoints>();
			node_reg.register_evaluator::<GetAbilityModifier>();
			node_reg.register_evaluator::<GetLevelInt>();
			node_reg.register_evaluator::<Math>();
			node_reg
		}

		#[test]
		fn value() -> anyhow::Result<()> {
			let doc = "mutator \"add_max_hit_points\" 5";
			let data = AddMaxHitPoints {
				id: None,
				value: Value::Fixed(5),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn evaluator() -> anyhow::Result<()> {
			let doc = "mutator \"add_max_hit_points\" \
			(Evaluator)\"get_ability_modifier\" (Ability)\"Constitution\"";
			let data = AddMaxHitPoints {
				id: None,
				value: Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn evaluator_math() -> anyhow::Result<()> {
			let doc = "
				|mutator \"add_max_hit_points\" (Evaluator)\"math\" \"Multiply\" id=\"Constitution x Levels\" {
				|    value (Evaluator)\"get_ability_modifier\" (Ability)\"Constitution\"
				|    value (Evaluator)\"get_level\"
				|}
			";
			let data = AddMaxHitPoints {
				id: Some("Constitution x Levels".into()),
				value: Value::Evaluated(
					Math {
						operation: MathOp::Multiply,
						minimum: None,
						maximum: None,
						values: vec![
							Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
							Value::Evaluated(GetLevelInt::default().into()),
						],
					}
					.into(),
				),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::{
			data::{character::Persistent, Ability, Bundle},
			evaluator::GetAbilityModifier,
		};

		fn character(mutator: AddMaxHitPoints) -> Character {
			let mut persistent = Persistent::default();
			persistent.bundles.push(
				Bundle {
					name: "TestMutator".into(),
					mutators: vec![mutator.into()],
					..Default::default()
				}
				.into(),
			);
			persistent.ability_scores[Ability::Constitution] = 14;
			Character::from(persistent)
		}

		#[test]
		fn fixed() {
			let character = character(AddMaxHitPoints {
				id: None,
				value: Value::Fixed(10),
			});
			assert_eq!(
				character.max_hit_points().sources(),
				&[("TestMutator".into(), 10),].into()
			);
			assert_eq!(character.max_hit_points().value(), 10);
		}

		#[test]
		fn evaluated() {
			let character = character(AddMaxHitPoints {
				id: None,
				value: Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
			});
			assert_eq!(
				character.max_hit_points().sources(),
				&[("TestMutator".into(), 2),].into()
			);
			assert_eq!(character.max_hit_points().value(), 2);
		}
	}
}
