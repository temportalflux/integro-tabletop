use super::EquipableEntry;
use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::{
		data::{
			action::{Action, ActivationKind, Attack, AttackCheckKind, AttackKindValue},
			Ability, DamageRoll, Feature, WeaponProficiency, roll::EvaluatedRoll,
		},
		evaluator::{self, IsProficientWith},
		Value,
	},
};
use std::str::FromStr;

mod damage;
pub use damage::*;
mod kind;
pub use kind::*;
mod property;
pub use property::*;
mod range;
pub use range::*;
mod restriction;
pub use restriction::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Weapon {
	pub kind: Kind,
	pub classification: String,
	pub damage: Option<WeaponDamage>,
	pub properties: Vec<Property>,
	pub range: Option<Range>,
}

impl Weapon {
	pub fn melee_reach(&self) -> Option<u32> {
		match &self.range {
			None => {
				let mut reach = 5;
				if self.properties.contains(&Property::Reach) {
					reach += 5;
				}
				Some(reach)
			}
			Some(_) => None,
		}
	}

	pub fn range(&self) -> Option<(u32, u32)> {
		match &self.range {
			None => {
				// melee weapons do not have a range/ranged attack - unless they have the thrown property
				self.properties.iter().find_map(|property| match property {
					Property::Thrown(short, long) => Some((*short, *long)),
					_ => None,
				})
			}
			Some(Range {
				short_range,
				long_range,
				..
			}) => Some((*short_range, *long_range)),
		}
	}

	pub fn attack_action(&self, entry: &EquipableEntry) -> Feature {
		// TODO: Attack should have properties for both melee and range to support the thrown property
		let attack_kind = match self.range {
			None => AttackKindValue::Melee {
				reach: self.melee_reach().unwrap(),
			},
			Some(Range {
				short_range,
				long_range,
				..
			}) => AttackKindValue::Ranged {
				short_dist: short_range,
				long_dist: long_range,
			},
		};
		// TODO: The ability modifier used for a melee weapon attack is Strength,
		// and the ability modifier used for a ranged weapon attack is Dexterity.
		// Weapons that have the finesse or thrown property break this rule.
		let attack_ability = match attack_kind {
			AttackKindValue::Melee { .. } => Ability::Strength,
			AttackKindValue::Ranged { .. } => Ability::Dexterity,
		};
		// TODO: Handle weapon properties
		Feature {
			name: entry.item.name.clone(),
			action: Some(Action {
				activation_kind: ActivationKind::Action,
				attack: Some(Attack {
					kind: Some(attack_kind),
					check: AttackCheckKind::AttackRoll {
						ability: attack_ability,
						proficient: Value::Evaluated(
							evaluator::Any(vec![
								IsProficientWith::Weapon(WeaponProficiency::Kind(self.kind)).into(),
								IsProficientWith::Weapon(WeaponProficiency::Classification(
									self.classification.clone(),
								))
								.into(),
							])
							.into(),
						),
					},
					area_of_effect: None,
					damage: self.damage.as_ref().map(|dmg| DamageRoll {
						roll: dmg.roll.map(|roll| EvaluatedRoll::from(roll)),
						base_bonus: dmg.bonus,
						damage_type: dmg.damage_type,
						..Default::default()
					}),
					weapon_kind: Some(self.kind),
				}),
				..Default::default()
			}),
			..Default::default()
		}
	}
}

impl FromKDL for Weapon {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let kind = Kind::from_str(node.get_str_req(ctx.consume_idx())?)?;
		let classification = node.get_str_req("class")?.to_owned();
		let damage = match node.query("scope() > damage")? {
			None => None,
			Some(node) => Some(WeaponDamage::from_kdl(node, &mut ctx.next_node())?),
		};
		let properties = {
			let mut props = Vec::new();
			for node in node.query_all("scope() > property")? {
				props.push(Property::from_kdl(node, &mut ctx.next_node())?);
			}
			props
		};
		let range = match node.query("scope() > range")? {
			None => None,
			Some(node) => Some(Range::from_kdl(node, &mut ctx.next_node())?),
		};
		Ok(Self {
			kind,
			classification,
			damage,
			properties,
			range,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;

	// TODO: Tests for generating an attack from a weapon

	mod from_kdl {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::dnd5e::data::{
				roll::{Die, Roll},
				DamageType,
			},
		};

		fn from_doc(doc: &str) -> anyhow::Result<Weapon> {
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > weapon")?
				.expect("missing weapon node");
			Weapon::from_kdl(node, &mut NodeContext::default())
		}

		#[test]
		fn simple() -> anyhow::Result<()> {
			let doc = "weapon \"Simple\" class=\"Handaxe\" {
				damage \"Slashing\" roll=\"1d6\"
				property \"Light\"
				property \"Thrown\" 20 60
			}";
			let expected = Weapon {
				kind: Kind::Simple,
				classification: "Handaxe".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll::from((1, Die::D6))),
					bonus: 0,
					damage_type: DamageType::Slashing,
				}),
				properties: vec![Property::Light, Property::Thrown(20, 60)],
				range: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn martial() -> anyhow::Result<()> {
			let doc = "weapon \"Martial\" class=\"Rapier\" {
				damage \"Piercing\" roll=\"1d8\"
				property \"Finesse\"
			}";
			let expected = Weapon {
				kind: Kind::Martial,
				classification: "Rapier".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll::from((1, Die::D8))),
					bonus: 0,
					damage_type: DamageType::Piercing,
				}),
				properties: vec![Property::Finesse],
				range: None,
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn ranged() -> anyhow::Result<()> {
			let doc = "weapon \"Martial\" class=\"CrossbowHand\" {
				damage \"Piercing\" roll=\"1d6\"
				property \"Light\"
				range 30 120 {
					ammunition
					loading
				}
			}";
			let expected = Weapon {
				kind: Kind::Martial,
				classification: "CrossbowHand".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll::from((1, Die::D6))),
					bonus: 0,
					damage_type: DamageType::Piercing,
				}),
				properties: vec![Property::Light],
				range: Some(Range {
					short_range: 30,
					long_range: 120,
					requires_ammunition: true,
					requires_loading: true,
				}),
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
