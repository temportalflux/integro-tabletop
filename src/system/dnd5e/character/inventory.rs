use super::{DerivedBuilder, State};
use crate::system::dnd5e::{
	criteria::BoxedCriteria,
	mutator::{self, AddSkillModifier, BoxedMutator},
	roll::{self, Die, Roll},
	Ability, Skill,
};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct Inventory {
	items_by_id: HashMap<Uuid, Item>,
	itemids_by_name: Vec<Uuid>,
}

impl Inventory {
	pub fn new() -> Self {
		Self {
			items_by_id: HashMap::new(),
			itemids_by_name: Vec::new(),
		}
	}

	fn find_name_for_id(&self, id: &Uuid) -> &String {
		&self.items_by_id.get(id).unwrap().name
	}

	pub fn insert(&mut self, item: Item) {
		let id = Uuid::new_v4();
		let search = self
			.itemids_by_name
			.binary_search_by(|id| self.find_name_for_id(id).cmp(&item.name));
		let idx = match search {
			// an item with the same name already exists at this index
			Ok(idx) => idx,
			// no item with the name exists, this is the index to insert to maintain sort-order
			Err(idx) => idx,
		};
		self.itemids_by_name.insert(idx, id.clone());
		self.items_by_id.insert(id, item);
	}

	pub fn remove(&mut self, id: &Uuid) -> Option<Item> {
		if let Ok(idx) = self.itemids_by_name.binary_search(id) {
			self.itemids_by_name.remove(idx);
		}
		self.items_by_id.remove(id)
	}

	pub fn items_without_ids(&self) -> std::collections::hash_map::Values<'_, Uuid, Item> {
		self.items_by_id.values()
	}

	pub fn items(&self) -> Vec<(&Uuid, &Item)> {
		self.itemids_by_name
			.iter()
			.map(|id| (id, self.items_by_id.get(&id).unwrap()))
			.collect()
	}

	pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Item> {
		self.items_by_id.get_mut(id)
	}
}

impl mutator::Container for Inventory {
	fn id(&self) -> Option<String> {
		Some("Inventory".into())
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		for item in self.items_by_id.values() {
			stats.apply_from(item);
		}
	}
}

#[derive(Clone, PartialEq, Default)]
pub struct Item {
	pub name: String,
	pub description: Option<String>,
	pub weight: u32,
	pub worth: u32,
	pub notes: String,
	pub kind: ItemKind,
}

impl Item {
	/// Returns true if the item has the capability to be equipped (i.e. it is a piece of equipment).
	pub fn is_equipable(&self) -> bool {
		match &self.kind {
			ItemKind::Equipment(_) => true,
			_ => false,
		}
	}

	/// Returs Ok if the item can currently be equipped, otherwise returns a user-displayable reason why it cannot be equipped.
	pub fn can_be_equipped(&self, state: &State) -> Result<(), String> {
		match &self.kind {
			ItemKind::Equipment(equipment) => equipment.can_be_equipped(state),
			_ => Ok(()),
		}
	}

	/// Returns true if the item is equipment and is currently equipped.
	pub fn is_equipped(&self) -> bool {
		match &self.kind {
			ItemKind::Equipment(equipment) => equipment.is_equipped,
			_ => false,
		}
	}

	pub fn set_equipped(&mut self, equipped: bool) {
		let ItemKind::Equipment(equipment) = &mut self.kind else { return; };
		equipment.is_equipped = equipped;
	}

	pub fn quantity(&self) -> u32 {
		match &self.kind {
			ItemKind::Simple { count } => *count,
			_ => 1,
		}
	}
}

impl mutator::Container for Item {
	fn id(&self) -> Option<String> {
		Some(self.name.clone())
	}

	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		if let ItemKind::Equipment(equipment) = &self.kind {
			stats.apply_from(equipment);
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum ItemKind {
	Simple { count: u32 },
	Equipment(Equipment),
}
impl Default for ItemKind {
	fn default() -> Self {
		Self::Simple { count: 1 }
	}
}

#[derive(Clone, PartialEq, Default)]
pub struct Equipment {
	pub is_equipped: bool,
	/// The criteria which must be met for this item to be equipped.
	pub criteria: Option<BoxedCriteria>,
	/// Passive modifiers applied while this item is equipped.
	pub modifiers: Vec<BoxedMutator>,
	/// If this item is armor, this is the armor data.
	pub armor: Option<Armor>,
	/// If this item is a shield, this is the AC bonus it grants.
	pub shield: Option<i32>,
	/// If this item is a weapon, tthis is the weapon data.
	pub weapon: Option<Weapon>,
	/// If this weapon can be attuned, this is the attunement data.
	pub attunement: Option<Attunement>,
}
impl mutator::Container for Equipment {
	fn apply_mutators<'c>(&self, stats: &mut DerivedBuilder<'c>) {
		if !self.is_equipped {
			return;
		}

		for modifier in &self.modifiers {
			stats.apply(modifier);
		}
	}
}
impl Equipment {
	/// Returs Ok if the item can currently be equipped, otherwise returns a user-displayable reason why it cannot be equipped.
	pub fn can_be_equipped(&self, state: &State) -> Result<(), String> {
		match &self.criteria {
			Some(criteria) => criteria.evaluate(state),
			None => Ok(()),
		}
	}
}

#[derive(Clone, PartialEq)]
pub struct Armor {
	pub kind: ArmorType,
	/// The minimum armor-class granted while this is equipped.
	pub base_score: u32,
	/// The ability modifier granted to AC.
	pub ability_modifier: Option<Ability>,
	/// The maximum ability modifier granted. If none, the modifier is unbounded.
	pub max_ability_bonus: Option<i32>,
	/// The minimum expected strength score to use this armor.
	/// If provided, characters with a value less than this are hindered (reduced speed).
	pub min_strength_score: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArmorType {
	Light,
	Medium,
	Heavy,
}

#[derive(Clone, PartialEq)]
pub struct Weapon {
	pub kind: WeaponType,
	pub damage: Roll,
	pub damage_type: String,
	pub properties: Vec<Property>,
	pub range: Option<WeaponRange>,
}
#[derive(Clone, PartialEq, Default)]
pub enum WeaponType {
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
pub struct WeaponRange {
	short_range: u32,
	long_range: u32,
	requires_ammunition: bool,
	requires_loading: bool,
}

#[derive(Clone, PartialEq, Default)]
pub struct Attunement {
	pub modifiers: Vec<BoxedMutator>,
}

#[allow(dead_code)]
fn items_demo() {
	let _armor_leather = Item {
		name: "Leather Armor".into(),
		description: None,
		weight: 10,
		worth: 1000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			armor: Some(Armor {
				kind: ArmorType::Light,
				base_score: 11,
				ability_modifier: Some(Ability::Dexterity),
				max_ability_bonus: None,
				min_strength_score: None,
			}),
			..Default::default()
		}),
	};
	let _armor_scale_mail = Item {
		name: "Scale Mail".into(),
		description: None,
		weight: 45,
		worth: 5000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![AddSkillModifier {
				skill: Skill::Stealth,
				modifier: roll::Modifier::Disadvantage,
				criteria: None,
			}
			.into()],
			armor: Some(Armor {
				kind: ArmorType::Medium,
				base_score: 14,
				ability_modifier: Some(Ability::Dexterity),
				max_ability_bonus: Some(2),
				min_strength_score: None,
			}),
			..Default::default()
		}),
	};
	let _armor_splint = Item {
		name: "Splint".into(),
		description: None,
		weight: 60,
		worth: 20000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			modifiers: vec![AddSkillModifier {
				skill: Skill::Stealth,
				modifier: roll::Modifier::Disadvantage,
				criteria: None,
			}
			.into()],
			armor: Some(Armor {
				kind: ArmorType::Heavy,
				base_score: 17,
				ability_modifier: None,
				max_ability_bonus: None,
				min_strength_score: Some(15),
			}),
			..Default::default()
		}),
	};
	let _club = Item {
		name: "Club".into(),
		description: None,
		weight: 2,
		worth: 10, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Simple,
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
	};
	let _dagger = Item {
		name: "Dagger".into(),
		description: None,
		weight: 1,
		worth: 200, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Simple,
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
	};
	let _greatclub = Item {
		name: "Greatclub".into(),
		description: None,
		weight: 10,
		worth: 20, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Simple,
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
	};
	let _quarterstaff = Item {
		name: "Quarterstaff".into(),
		description: None,
		weight: 4,
		worth: 20, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Simple,
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
	};
	let _crossbow_light = Item {
		name: "Crossbow (Light)".into(),
		description: None,
		weight: 5,
		worth: 2500, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Simple,
				damage: Roll {
					amount: 1,
					die: Die::D8,
				},
				damage_type: "piercing".into(),
				properties: vec![Property::TwoHanded],
				range: Some(WeaponRange {
					short_range: 80,
					long_range: 320,
					requires_ammunition: true,
					requires_loading: true,
				}),
			}),
			..Default::default()
		}),
	};
	let _halberd = Item {
		name: "Halberd".into(),
		description: None,
		weight: 6,
		worth: 2000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Martial,
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
	};
	let _longbow = Item {
		name: "Halberd".into(),
		description: None,
		weight: 18,
		worth: 5000, // in copper
		notes: "".into(),
		kind: ItemKind::Equipment(Equipment {
			weapon: Some(Weapon {
				kind: WeaponType::Martial,
				damage: Roll {
					amount: 1,
					die: Die::D8,
				},
				damage_type: "slashing".into(),
				properties: vec![Property::TwoHanded, Property::Heavy],
				range: Some(WeaponRange {
					short_range: 150,
					long_range: 600,
					requires_ammunition: true,
					requires_loading: false,
				}),
			}),
			..Default::default()
		}),
	};
}
