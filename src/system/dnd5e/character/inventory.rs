use crate::system::dnd5e::{
	modifier::BoxedModifier,
	roll::{Die, Roll},
	Feature,
};

#[derive(Clone, PartialEq)]
pub struct Inventory {
	pub items: Vec<(CustomItem, u32)>,
}

impl Inventory {
	pub fn new() -> Self {
		Self { items: Vec::new() }
	}
}

#[derive(Clone, PartialEq)]
pub struct CustomItem {
	pub name: String,
	pub description: Option<String>,
	pub weight: u32,
	pub worth: u32,
	pub notes: String,
	pub kind: ItemKind,
}

#[derive(Clone, PartialEq)]
pub enum ItemKind {
	Simple { count: u32 },
	Equipment(Equipment),
}

#[derive(Clone, PartialEq, Default)]
pub struct Equipment {
	is_equipped: bool,
	modifiers: Vec<BoxedModifier>,
	features: Vec<Feature>,
	armor: Option<Armor>,
	weapon: Option<Weapon>,
	attunement: Option<Attunement>,
}

#[derive(Clone, PartialEq, Default)]
pub struct Weapon {
	/*
	Weapon (https://www.dndbeyond.com/sources/basic-rules/equipment#Weapons):
	- type (melee, ranged)
	- type 2 (martial, simple, spell)
	- archetype (for proficiency)
	- reach / range
	- damage (roll)
	- damage type
	- properties (finesse, light, thrown, etc)
	*/
}

#[derive(Clone, PartialEq, Default)]
pub struct Armor {}

#[derive(Clone, PartialEq, Default)]
pub struct Attunement {
	pub modifiers: Vec<BoxedModifier>,
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

fn items_demo() {
	let _armor_leather = CustomItem {
		name: "Leather Armor".into(),
		description: None,
		weight: 10,
		worth: 1000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			armor: Some(Armor {
				/*
				kind: ArmorType::Light,
				ac_base: 11,
				ac_modifier: Some(Ability::Dexterity),
				ac_modifier_limit: None,
				*/
			}),
			..Default::default()
		}),
	};
	let _armor_scale_mail = CustomItem {
		name: "Scale Mail".into(),
		description: None,
		weight: 45,
		worth: 5000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![
				// disadvantage on stealth
			],
			armor: Some(Armor {
				/*
				kind: ArmorType::Medium,
				ac_base: 14,
				ac_modifier: Some(Ability::Dexterity),
				ac_modifier_limit: Some(2),
				*/
			}),
			..Default::default()
		}),
	};
	let _armor_splint = CustomItem {
		name: "Splint".into(),
		description: None,
		weight: 60,
		worth: 20000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![
				// disadvantage on stealth
			],
			armor: Some(Armor {
				/*
				kind: ArmorType::Heavy,
				ac_base: 17,
				ac_modifier: None,
				ac_modifier_limit: None,
				minimum_strength: Some(15),
				*/
			}),
			..Default::default()
		}),
	};
	let _club = CustomItem {
		name: "Club".into(),
		description: None,
		weight: 2,
		worth: 10, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Simple,
				damage: Roll { amount: 1, die: Die::D4 },
				damage_type: "bludgeoning".into(),
				is_light: true,
				properties: vec![Property::Light],
				*/
			}),
			..Default::default()
		}),
	};
	let _dagger = CustomItem {
		name: "Dagger".into(),
		description: None,
		weight: 1,
		worth: 200, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Simple,
				damage: Roll { amount: 1, die: Die::D4 },
				damage_type: "piercing".into(),
				properties: vec![Property::Light, Property::Finesse, Property::Thrown(20, 60)],
				*/
			}),
			..Default::default()
		}),
	};
	let _greatclub = CustomItem {
		name: "Greatclub".into(),
		description: None,
		weight: 10,
		worth: 20, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Simple,
				damage: Roll { amount: 1, die: Die::D8 },
				damage_type: "bludgeoning".into(),
				properties: vec![Property::TwoHanded],
				*/
			}),
			..Default::default()
		}),
	};
	let _quarterstaff = CustomItem {
		name: "Quarterstaff".into(),
		description: None,
		weight: 4,
		worth: 20, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Simple,
				damage: Roll { amount: 1, die: Die::D6 },
				damage_type: "bludgeoning".into(),
				properties: vec![Property::Versatile(Roll { amount: 1, die: Die::D8 })],
				*/
			}),
			..Default::default()
		}),
	};
	let _crossbow_light = CustomItem {
		name: "Crossbow (Light)".into(),
		description: None,
		weight: 5,
		worth: 2500, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Simple,
				damage: Roll { amount: 1, die: Die::D8 },
				damage_type: "piercing".into(),
				properties: vec![Property::TwoHanded],
				ranged: Some(WeaponRange {
					short_range: 80,
					long_range: 320,
					properties: vec![RangeProperty::Ammunition, RangeProperty::Loading],
				}),
				*/
			}),
			..Default::default()
		}),
	};
	let _halberd = CustomItem {
		name: "Halberd".into(),
		description: None,
		weight: 6,
		worth: 2000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Martial,
				damage: Roll { amount: 1, die: Die::D10 },
				damage_type: "slashing".into(),
				properties: vec![Property::TwoHanded, Property::Heavy, Property::Reach],
				ranged: None,
				*/
			}),
			..Default::default()
		}),
	};
	let _longbow = CustomItem {
		name: "Halberd".into(),
		description: None,
		weight: 18,
		worth: 5000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				/*
				classification: Classification::Martial,
				damage: Roll { amount: 1, die: Die::D8 },
				damage_type: "slashing".into(),
				properties: vec![Property::TwoHanded, Property::Heavy],
				ranged: Some(WeaponRange {
					short_range: 150,
					long_range: 600,
					properties: vec![RangeProperty::Ammunition],
				}),
				*/
			}),
			..Default::default()
		}),
	};
}
