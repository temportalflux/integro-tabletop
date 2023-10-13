use crate::kdl_ext::NodeContext;
use crate::{
	system::dnd5e::data::{action::AttackQuery, character::Character},
	utility::Evaluator,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

/// Checks if the character has an action with an attack that matches a restriction.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasAttack {
	min: usize,
	max: Option<usize>,
	restriction: AttackQuery,
}

crate::impl_trait_eq!(HasAttack);
kdlize::impl_kdl_node!(HasAttack, "has_attack");

impl Evaluator for HasAttack {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(match (&self.min, &self.max) {
			(1, None) => format!("you have a weapon equipped which: {}", self.restriction),
			(1, Some(max)) => format!(
				"you have no more than {max} weapons equipped which: {}",
				self.restriction
			),
			(min, None) => format!("you have at least {min} weapons equipped which: {}", self.restriction),
			(min, Some(max)) => format!(
				"you have at least {min}, and no more than {max}, weapons equipped which: {}",
				self.restriction
			),
		})
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		let mut count = 0usize;
		for (_source, feature) in character.features().iter_all() {
			let Some(action) = &feature.action else {
				continue;
			};
			let Some(attack) = &action.attack else {
				continue;
			};
			if !self.restriction.is_attack_valid(attack) {
				continue;
			}

			count += 1;
			match &self.max {
				// success, something was found
				None => return Ok(()),
				Some(max) if count > *max => return Err("Equipment max exceeded".into()),
				Some(_) => {}
			}
		}
		if count >= self.min {
			Ok(())
		} else {
			Err("Equipped weapons not found".into())
		}
	}
}

impl FromKdl<NodeContext> for HasAttack {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let min = node.get_i64_opt("min")?.unwrap_or(1) as usize;
		let max = node.get_i64_opt("max")?.map(|v| v as usize);
		let restriction = AttackQuery::from_kdl(node)?;
		Ok(Self { min, max, restriction })
	}
}

impl AsKdl for HasAttack {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if self.min > 1 {
			node.push_entry(("min", self.min as i64));
		}
		if let Some(max) = &self.max {
			node.push_entry(("max", *max as i64));
		}
		node += self.restriction.as_kdl();
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
			system::dnd5e::{
				data::{action::AttackKind, item::weapon},
				evaluator::test::test_utils,
			},
		};

		test_utils!(HasAttack);

		#[test]
		fn any() -> anyhow::Result<()> {
			let doc = "evaluator \"has_attack\"";
			let data = HasAttack {
				min: 1,
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn single() -> anyhow::Result<()> {
			let doc = "evaluator \"has_attack\" max=1";
			let data = HasAttack {
				min: 1,
				max: Some(1),
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn restricted() -> anyhow::Result<()> {
			let doc = "
				|evaluator \"has_attack\" min=3 {
				|    weapon \"Simple\" \"Martial\"
				|    attack \"Melee\"
				|}
			";
			let data = HasAttack {
				min: 3,
				max: None,
				restriction: AttackQuery {
					weapon_kind: weapon::Kind::Simple | weapon::Kind::Martial,
					attack_kind: AttackKind::Melee.into(),
					..Default::default()
				},
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
