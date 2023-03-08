use crate::system::dnd5e::data::{
	item::{
		equipment::Equipment,
		weapon::{self, Property, Weapon, WeaponDamage},
		Item, ItemKind,
	},
	roll::{Die, Roll}, action::DamageType,
};

pub fn club() -> Item {
	Item {
		name: "Club".into(),
		description: None,
		weight: 2.0,
		worth: 10, // in copper
		notes: None,
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Club".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll {
						amount: 1,
						die: Die::D4,
					}),
					damage_type: DamageType::Bludgeoning,
					..Default::default()
				}),
				properties: vec![Property::Light],
				range: None,
			}),
			..Default::default()
		}),
		..Default::default()
	}
}

pub fn dagger() -> Item {
	Item {
		name: "Dagger".into(),
		description: None,
		weight: 1.0,
		worth: 200, // in copper
		notes: None,
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Dagger".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll {
						amount: 1,
						die: Die::D4,
					}),
					damage_type: DamageType::Piercing,
					..Default::default()
				}),
				properties: vec![Property::Light, Property::Finesse, Property::Thrown(20, 60)],
				range: None,
			}),
			..Default::default()
		}),
		..Default::default()
	}
}

pub fn greatclub() -> Item {
	Item {
		name: "Greatclub".into(),
		description: None,
		weight: 10.0,
		worth: 20, // in copper
		notes: None,
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Greatclub".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll {
						amount: 1,
						die: Die::D8,
					}),
					damage_type: DamageType::Bludgeoning,
					..Default::default()
				}),
				properties: vec![Property::TwoHanded],
				range: None,
			}),
			..Default::default()
		}),
		..Default::default()
	}
}

pub fn quarterstaff() -> Item {
	Item {
		name: "Quarterstaff".into(),
		description: None,
		weight: 4.0,
		worth: 20, // in copper
		notes: None,
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Quarterstaff".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll {
						amount: 1,
						die: Die::D6,
					}),
					damage_type: DamageType::Bludgeoning,
					..Default::default()
				}),
				properties: vec![Property::Versatile(Roll {
					amount: 1,
					die: Die::D8,
				})],
				range: None,
			}),
			..Default::default()
		}),
		..Default::default()
	}
}

pub fn crossbow_light() -> Item {
	Item {
		name: "Crossbow (Light)".into(),
		description: None,
		weight: 5.0,
		worth: 2500, // in copper
		notes: None,
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "CrossbowLight".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll {
						amount: 1,
						die: Die::D8,
					}),
					damage_type: DamageType::Piercing,
					..Default::default()
				}),
				properties: vec![Property::TwoHanded],
				range: Some(weapon::Range {
					short_range: 80,
					long_range: 320,
					requires_ammunition: true,
					requires_loading: true,
				}),
			}),
			..Default::default()
		}),
		..Default::default()
	}
}

pub fn halberd() -> Item {
	Item {
		name: "Halberd".into(),
		description: None,
		weight: 6.0,
		worth: 2000, // in copper
		notes: None,
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Martial,
				classification: "Halberd".into(),
				damage: Some(WeaponDamage {
					roll: Some(Roll {
						amount: 1,
						die: Die::D10,
					}),
					damage_type: DamageType::Slashing,
					..Default::default()
				}),
				properties: vec![Property::TwoHanded, Property::Heavy, Property::Reach],
				range: None,
			}),
			..Default::default()
		}),
		..Default::default()
	}
}
