use std::collections::HashSet;

use super::EquipableEntry;
use crate::system::dnd5e::{
	action::{
		Action, ActionSource, ActivationKind, Attack, AttackCheckKind, AttackKind, AttackKindValue,
		DamageRoll,
	},
	character::WeaponProficiency,
	evaluator::{self, IsProficientWith},
	roll::Roll,
	Ability, Value,
};

#[derive(Clone, PartialEq)]
pub struct Weapon {
	pub kind: Kind,
	pub classification: String,
	pub damage: Roll,
	pub damage_type: String,
	pub properties: Vec<Property>,
	pub range: Option<Range>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
	#[default]
	Simple,
	Martial,
}
#[derive(Clone, PartialEq)]
pub enum Property {
	Light,   // used by two handed fighting feature
	Finesse, // melee weapons use strength, ranged use dex, finesse take the better of either modifier
	Heavy,   // small or tiny get disadvantage on attack rolls when using this weapon
	Reach,
	TwoHanded,
	Thrown(u32, u32),
	Versatile(Roll),
}

#[derive(Clone, PartialEq, Default)]
pub struct Range {
	pub short_range: u32,
	pub long_range: u32,
	pub requires_ammunition: bool,
	pub requires_loading: bool,
}

#[derive(Clone, PartialEq, Default)]
pub struct Restriction {
	pub weapon_kind: HashSet<Kind>,
	pub attack_kind: HashSet<AttackKind>,
	pub ability: HashSet<Ability>,
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
				damage_roll: DamageRoll {
					roll: Some(self.damage),
					..Default::default()
				},
				damage_type: self.damage_type.clone(),
			}),
			..Default::default()
		}
	}
}