use crate::system::dnd5e::data::{
	item::{
		equipment::Equipment,
		weapon::{self, Property, Weapon},
		Item, ItemKind,
	},
	roll::{Die, Roll},
};

pub fn club() -> Item {
	Item {
		name: "Club".into(),
		description: None,
		weight: 2,
		worth: 10, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Club".into(),
				damage: Roll {
					amount: 1,
					die: Die::D4,
				},
				damage_type: "bludgeoning".into(),
				properties: vec![Property::Light],
				range: None,
			}),
			..Default::default()
		}),
	}
}

pub fn dagger() -> Item {
	Item {
		name: "Dagger".into(),
		description: None,
		weight: 1,
		worth: 200, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Dagger".into(),
				damage: Roll {
					amount: 1,
					die: Die::D4,
				},
				damage_type: "piercing".into(),
				properties: vec![Property::Light, Property::Finesse, Property::Thrown(20, 60)],
				range: None,
			}),
			..Default::default()
		}),
	}
}

pub fn greatclub() -> Item {
	Item {
		name: "Greatclub".into(),
		description: None,
		weight: 10,
		worth: 20, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Greatclub".into(),
				damage: Roll {
					amount: 1,
					die: Die::D8,
				},
				damage_type: "bludgeoning".into(),
				properties: vec![Property::TwoHanded],
				range: None,
			}),
			..Default::default()
		}),
	}
}

pub fn quarterstaff() -> Item {
	Item {
		name: "Quarterstaff".into(),
		description: None,
		weight: 4,
		worth: 20, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "Quarterstaff".into(),
				damage: Roll {
					amount: 1,
					die: Die::D6,
				},
				damage_type: "bludgeoning".into(),
				properties: vec![Property::Versatile(Roll {
					amount: 1,
					die: Die::D8,
				})],
				range: None,
			}),
			..Default::default()
		}),
	}
}

pub fn crossbow_light() -> Item {
	Item {
		name: "Crossbow (Light)".into(),
		description: None,
		weight: 5,
		worth: 2500, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Simple,
				classification: "CrossbowLight".into(),
				damage: Roll {
					amount: 1,
					die: Die::D8,
				},
				damage_type: "piercing".into(),
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
	}
}

pub fn halberd() -> Item {
	Item {
		name: "Halberd".into(),
		description: None,
		weight: 6,
		worth: 2000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: weapon::Kind::Martial,
				classification: "Halberd".into(),
				damage: Roll {
					amount: 1,
					die: Die::D10,
				},
				damage_type: "slashing".into(),
				properties: vec![Property::TwoHanded, Property::Heavy, Property::Reach],
				range: None,
			}),
			..Default::default()
		}),
	}
}
