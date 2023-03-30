use super::EquipableEntry;
use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::{
		data::{
			action::{
				Action, ActionSource, ActivationKind, Attack, AttackCheckKind, AttackKind,
				AttackKindValue,
			},
			evaluator::{self, IsProficientWith},
			roll::Roll,
			Ability, DamageRoll, DamageType, WeaponProficiency,
		},
		Value,
	},
	GeneralError,
};
use std::{collections::HashSet, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Restriction {
	pub weapon_kind: HashSet<Kind>,
	pub attack_kind: HashSet<AttackKind>,
	pub ability: HashSet<Ability>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Weapon {
	pub kind: Kind,
	pub classification: String,
	pub damage: Option<WeaponDamage>,
	pub properties: Vec<Property>,
	pub range: Option<Range>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
	#[default]
	Simple,
	Martial,
}
impl ToString for Kind {
	fn to_string(&self) -> String {
		match self {
			Self::Simple => "Simple",
			Self::Martial => "Martial",
		}
		.to_owned()
	}
}
impl FromStr for Kind {
	type Err = GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Simple" => Ok(Self::Simple),
			"Martial" => Ok(Self::Martial),
			_ => Err(GeneralError(format!(
				"Invalid weapon kind {s:?}, expected Simple or Martial."
			))),
		}
	}
}

impl Weapon {
	pub fn attack_action(&self, entry: &EquipableEntry) -> Action {
		let attack_kind = match self.range {
			None => AttackKindValue::Melee { reach: 5 },
			Some(Range {
				short_range,
				long_range,
				..
			}) => AttackKindValue::Ranged {
				short_dist: short_range,
				long_dist: long_range,
				kind: None,
			},
		};
		// TODO: The ability modifier used for a melee weapon attack is Strength,
		// and the ability modifier used for a ranged weapon attack is Dexterity.
		// Weapons that have the finesse or thrown property break this rule.
		let attack_ability = match attack_kind {
			AttackKindValue::Melee { .. } => Ability::Strength,
			AttackKindValue::Ranged { .. } => Ability::Dexterity,
		};
		Action {
			name: entry.item.name.clone(),
			activation_kind: ActivationKind::Action,
			source: Some(ActionSource::Item(entry.id.clone())),
			attack: Some(Attack {
				kind: attack_kind,
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
					roll: dmg.roll,
					base_bonus: dmg.bonus,
					damage_type: dmg.damage_type,
					..Default::default()
				}),
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

#[derive(Clone, PartialEq, Default, Debug)]
pub struct WeaponDamage {
	pub roll: Option<Roll>,
	pub bonus: i32,
	pub damage_type: DamageType,
}

impl FromKDL for WeaponDamage {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let roll = match node.get_str_opt("roll")? {
			Some(roll_str) => Some(Roll::from_str(roll_str)?),
			None => None,
		};
		let base = node.get_i64_opt("base")?.unwrap_or(0) as i32;
		let damage_type = DamageType::from_str(node.get_str_req(ctx.consume_idx())?)?;
		Ok(Self {
			roll,
			bonus: base,
			damage_type,
		})
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum Property {
	Light,   // used by two handed fighting feature
	Finesse, // melee weapons use strength, ranged use dex, finesse take the better of either modifier
	Heavy,   // small or tiny get disadvantage on attack rolls when using this weapon
	Reach, // This weapon adds 5 feet to your reach when you attack with it, as well as when determining your reach for opportunity attacks with it.
	TwoHanded,
	Thrown(u32, u32),
	Versatile(Roll),
}
impl FromKDL for Property {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Light" => Ok(Self::Light),
			"Finesse" => Ok(Self::Finesse),
			"Heavy" => Ok(Self::Heavy),
			"Reach" => Ok(Self::Reach),
			"TwoHanded" => Ok(Self::TwoHanded),
			"Thrown" => {
				let short = node.get_i64_req(ctx.consume_idx())? as u32;
				let long = node.get_i64_req(ctx.consume_idx())? as u32;
				Ok(Self::Thrown(short, long))
			}
			"Versatile" => {
				let roll = Roll::from_str(node.get_str_req(ctx.consume_idx())?)?;
				Ok(Self::Versatile(roll))
			}
			name => Err(GeneralError(format!("Unrecognized weapon property {name:?}")).into()),
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Range {
	pub short_range: u32,
	pub long_range: u32,
	pub requires_ammunition: bool,
	pub requires_loading: bool,
}

impl FromKDL for Range {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let short_range = node.get_i64_req(ctx.consume_idx())? as u32;
		let long_range = node.get_i64_req(ctx.consume_idx())? as u32;
		let requires_ammunition = node.query("scope() > ammunition")?.is_some();
		let requires_loading = node.query("scope() > loading")?.is_some();
		Ok(Self {
			short_range,
			long_range,
			requires_ammunition,
			requires_loading,
		})
	}
}

// TODO: Test just all of it
