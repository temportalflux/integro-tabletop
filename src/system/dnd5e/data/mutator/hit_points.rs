use crate::{
	kdl_ext::{FromKDL, NodeExt, ValueExt},
	system::dnd5e::{data::character::Character, Value},
	utility::{Dependencies, Evaluator, Mutator},
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

	// TODO: mutator description add_max_hit_points

	fn dependencies(&self) -> Dependencies {
		self.value.dependencies()
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
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let value = Value::from_kdl(node, node.entry_req(ctx.consume_idx())?, ctx, |value| {
			Ok(value.as_i64_req()? as i32)
		})?;
		Ok(Self { id: None, value })
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod from_kdl {
		use super::*;
		use crate::system::{
			core::NodeRegistry,
			dnd5e::{
				data::{evaluator::GetAbilityModifier, Ability},
				BoxedMutator,
			},
		};

		fn from_doc(doc: &str) -> anyhow::Result<BoxedMutator> {
			let mut node_reg = NodeRegistry::default();
			node_reg.register_mutator::<AddMaxHitPoints>();
			node_reg
				.register_evaluator::<crate::system::dnd5e::data::evaluator::GetAbilityModifier>();
			node_reg.parse_kdl_mutator(doc)
		}

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
				value: Value::Evaluated(GetAbilityModifier(Ability::Constitution).into()),
			};
			assert_eq!(from_doc(doc)?, expected.into());
			Ok(())
		}
	}

	mod mutate {
		use super::*;
		use crate::system::dnd5e::data::{
			character::Persistent, evaluator::GetAbilityModifier, Ability, Feature,
		};

		fn character(mutator: AddMaxHitPoints) -> Character {
			let mut persistent = Persistent::default();
			persistent.feats.push(
				Feature {
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
				&[
					("Constitution x Levels".into(), 0),
					("TestMutator".into(), 10),
				]
				.into()
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
				&[
					("Constitution x Levels".into(), 0),
					("TestMutator".into(), 2),
				]
				.into()
			);
			assert_eq!(character.max_hit_points().value(), 2);
		}
	}
}
