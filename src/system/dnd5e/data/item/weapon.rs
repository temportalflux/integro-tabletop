use crate::kdl_ext::NodeContext;
use crate::system::dnd5e::{
	data::{
		action::{Action, ActivationKind, Attack, AttackCheckKind, AttackKind, AttackKindValue},
		item::container::item::EquipableEntry,
		roll::EvaluatedRoll,
		Ability, DamageRoll, Feature, WeaponProficiency,
	},
	evaluator::{self, IsProficientWith},
	Value,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::collections::HashMap;

mod damage;
pub use damage::*;
mod kind;
pub use kind::*;
mod property;
pub use property::*;
mod range;
pub use range::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Weapon {
	pub kind: Kind,
	pub classification: String,
	pub damage: Option<WeaponDamage>,
	pub properties: Vec<Property>,
	pub range: Option<Range>,
}

impl Weapon {
	pub fn to_metadata(self) -> serde_json::Value {
		let mut contents: HashMap<&'static str, serde_json::Value> = [("kind", self.kind.to_string().into())].into();
		if !self.classification.is_empty() {
			contents.insert("classification", self.classification.into());
		}
		if let Some(damage) = self.damage {
			contents.insert("damage_type", damage.damage_type.to_string().into());
		}
		contents.insert("has_range", self.range.is_some().into());
		serde_json::json!(contents)
	}

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

	pub fn attack_kind(&self) -> AttackKind {
		match &self.range {
			None => AttackKind::Melee,
			Some(_) => AttackKind::Ranged,
		}
	}

	pub fn attack_ability(&self) -> Ability {
		match self.attack_kind() {
			AttackKind::Melee => Ability::Strength,
			AttackKind::Ranged => Ability::Dexterity,
		}
	}

	pub fn attack_action(&self, entry: &EquipableEntry) -> Feature {
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
		Feature {
			name: entry.item.name.clone(),
			action: Some(Action {
				activation_kind: ActivationKind::Action,
				attack: Some(Attack {
					kind: Some(attack_kind),
					check: AttackCheckKind::AttackRoll {
						ability: self.attack_ability(),
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
					classification: Some(self.classification.clone()),
					properties: self.properties.clone(),
				}),
				..Default::default()
			}),
			..Default::default()
		}
	}
}

impl FromKdl<NodeContext> for Weapon {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = node.next_str_req_t::<Kind>()?;
		let classification = node.get_str_req("class")?.to_owned();
		let damage = node.query_opt_t::<WeaponDamage>("scope() > damage")?;
		let properties = {
			let mut props = Vec::new();
			for mut node in &mut node.query_all("scope() > property")? {
				props.push(Property::from_kdl(&mut node)?);
			}
			props
		};
		let range = node.query_opt_t::<Range>("scope() > range")?;
		Ok(Self {
			kind,
			classification,
			damage,
			properties,
			range,
		})
	}
}

impl AsKdl for Weapon {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.push_entry(self.kind.to_string());
		node.push_entry(("class", self.classification.clone()));
		node.push_child_t(("damage", &self.damage));
		node.push_children_t(("property", self.properties.iter()));
		node.push_child_t(("range", &self.range));
		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	// TODO: Tests for generating an attack from a weapon

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::dnd5e::data::{
				roll::{Die, Roll},
				DamageType,
			},
		};

		static NODE_NAME: &str = "weapon";

		#[test]
		fn simple() -> anyhow::Result<()> {
			let doc = "
				|weapon \"Simple\" class=\"Handaxe\" {
				|    damage \"Slashing\" roll=\"1d6\"
				|    property \"Light\"
				|    property \"Thrown\" 20 60
				|}
			";
			let data = Weapon {
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
			assert_eq_fromkdl!(Weapon, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn martial() -> anyhow::Result<()> {
			let doc = "
				|weapon \"Martial\" class=\"Rapier\" {
				|    damage \"Piercing\" roll=\"1d8\"
				|    property \"Finesse\"
				|}
			";
			let data = Weapon {
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
			assert_eq_fromkdl!(Weapon, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn ranged() -> anyhow::Result<()> {
			let doc = "
				|weapon \"Martial\" class=\"CrossbowHand\" {
				|    damage \"Piercing\" roll=\"1d6\"
				|    property \"Light\"
				|    range 30 120 {
				|        ammunition
				|        loading
				|    }
				|}
			";
			let data = Weapon {
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
			assert_eq_fromkdl!(Weapon, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
