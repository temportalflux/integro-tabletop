use crate::system::dnd5e::{
	action::{
		Action, ActionSource, ActivationKind, Attack, AttackCheckKind, AttackKind, AttackKindValue,
	},
	character::WeaponProficiency,
	evaluator::IsProficientWith,
	roll::Roll,
	Ability, Value,
};
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct Weapon {
	pub kind: Kind,
	pub classification: String,
	pub damage: Roll,
	pub damage_type: String,
	pub properties: Vec<Property>,
	pub range: Option<Range>,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
	pub weapon_kind: Vec<Kind>,
	pub attack_kind: Vec<AttackKind>,
	pub ability: Vec<Ability>,
}

impl Weapon {
	pub fn attack_action(&self, name: String, id: &Uuid) -> Action {
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
			name,
			activation_kind: ActivationKind::Action,
			source: Some(ActionSource::Item(*id)),
			attack: Some(Attack {
				kind: attack_kind,
				check: AttackCheckKind::AttackRoll {
					ability: attack_ability,
					proficient: Value::Evaluated(
						IsProficientWith::Weapon(WeaponProficiency::Classification(
							self.classification.clone(),
						))
						.into(),
					),
				},
				area_of_effect: None,
				damage_roll: (Some(self.damage), Value::Fixed(0)),
				damage_type: self.damage_type.clone(),
			}),
			..Default::default()
		}
	}
}
