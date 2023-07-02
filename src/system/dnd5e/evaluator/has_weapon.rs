use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::data::{
		character::Character,
		item::{self, weapon},
	},
	utility::Evaluator,
};

/// Checks if the character has a weapon equipped.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasWeaponEquipped {
	min: usize,
	max: Option<usize>,
	restriction: weapon::Restriction,
}

crate::impl_trait_eq!(HasWeaponEquipped);
crate::impl_kdl_node!(HasWeaponEquipped, "has_weapon_equipped");

impl Evaluator for HasWeaponEquipped {
	type Context = Character;
	type Item = Result<(), String>;

	fn description(&self) -> Option<String> {
		Some(match (&self.min, &self.max) {
			(1, None) => format!("you have a weapon equipped which: {}", self.restriction),
			(1, Some(max)) => format!(
				"you have no more than {max} weapons equipped which: {}",
				self.restriction
			),
			(min, None) => format!(
				"you have at least {min} weapons equipped which: {}",
				self.restriction
			),
			(min, Some(max)) => format!(
				"you have at least {min}, and no more than {max}, weapons equipped which: {}",
				self.restriction
			),
		})
	}

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		let mut count = 0usize;
		for entry in character.inventory().entries() {
			if !entry.is_equipped {
				continue;
			}
			let item::Kind::Equipment(equipment) = &entry.item.kind else { continue; };
			let Some(weapon) = &equipment.weapon else { continue; };
			if !self.restriction.does_weapon_meet(weapon) {
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

impl FromKDL for HasWeaponEquipped {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let min = node.get_i64_opt("min")?.unwrap_or(1) as usize;
		let max = node.get_i64_opt("max")?.map(|v| v as usize);
		let restriction = weapon::Restriction::from_kdl(node, ctx)?;
		Ok(Self {
			min,
			max,
			restriction,
		})
	}
}

impl AsKdl for HasWeaponEquipped {
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
		use crate::{kdl_ext::test_utils::*, system::dnd5e::{evaluator::test::test_utils, data::action::AttackKind}};

		test_utils!(HasWeaponEquipped);

		#[test]
		fn any() -> anyhow::Result<()> {
			let doc = "evaluator \"has_weapon_equipped\"";
			let data = HasWeaponEquipped {
				min: 1,
				..Default::default()
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn single() -> anyhow::Result<()> {
			let doc = "evaluator \"has_weapon_equipped\" max=1";
			let data = HasWeaponEquipped {
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
				|evaluator \"has_weapon_equipped\" min=3 {
				|    weapon \"Simple\" \"Martial\"
				|    attack \"Melee\"
				|}
			";
			let data = HasWeaponEquipped {
				min: 3,
				max: None,
				restriction: weapon::Restriction {
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
