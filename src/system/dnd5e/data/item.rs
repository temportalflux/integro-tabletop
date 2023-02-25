use super::mutator::AddAction;
use crate::system::dnd5e::{data::character::Character, mutator};
use std::collections::HashMap;
use uuid::Uuid;

pub mod armor;
pub mod equipment;
pub mod weapon;

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
	pub fn can_be_equipped(&self, state: &Character) -> Result<(), String> {
		match &self.kind {
			ItemKind::Equipment(equipment) => equipment.can_be_equipped(state),
			_ => Ok(()),
		}
	}

	pub fn quantity(&self) -> u32 {
		match &self.kind {
			ItemKind::Simple { count } => *count,
			_ => 1,
		}
	}
}

#[derive(Clone, PartialEq)]
pub enum ItemKind {
	Simple { count: u32 },
	Equipment(equipment::Equipment),
}

impl Default for ItemKind {
	fn default() -> Self {
		Self::Simple { count: 1 }
	}
}

#[derive(Clone, PartialEq)]
pub struct EquipableEntry {
	pub id: Uuid,
	pub item: Item,
	pub is_equipped: bool,
}
impl mutator::Container for EquipableEntry {
	fn id(&self) -> Option<String> {
		Some(self.item.name.clone())
	}

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		if let ItemKind::Equipment(equipment) = &self.item.kind {
			if self.is_equipped {
				stats.apply_from(equipment);
				if let Some(weapon) = &equipment.weapon {
					stats.apply(&AddAction(weapon.attack_action(self)).into());
				}
			}
		}
	}
}

#[derive(Clone, PartialEq, Default)]
pub struct Inventory {
	items_by_id: HashMap<Uuid, EquipableEntry>,
	itemids_by_name: Vec<Uuid>,
}

impl Inventory {
	pub fn new() -> Self {
		Self {
			items_by_id: HashMap::new(),
			itemids_by_name: Vec::new(),
		}
	}

	pub fn get_item(&self, id: &Uuid) -> Option<&Item> {
		self.items_by_id.get(id).map(|entry| &entry.item)
	}

	pub fn insert(&mut self, item: Item) {
		let id = Uuid::new_v4();
		let search = self
			.itemids_by_name
			.binary_search_by(|id| self.get_item(id).unwrap().name.cmp(&item.name));
		let idx = match search {
			// an item with the same name already exists at this index
			Ok(idx) => idx,
			// no item with the name exists, this is the index to insert to maintain sort-order
			Err(idx) => idx,
		};
		self.itemids_by_name.insert(idx, id.clone());
		self.items_by_id.insert(
			id.clone(),
			EquipableEntry {
				id,
				item,
				is_equipped: false,
			},
		);
	}

	pub fn remove(&mut self, id: &Uuid) -> Option<Item> {
		if let Ok(idx) = self.itemids_by_name.binary_search(id) {
			self.itemids_by_name.remove(idx);
		}
		self.items_by_id.remove(id).map(|entry| entry.item)
	}

	pub fn entries(&self) -> impl Iterator<Item = &EquipableEntry> {
		self.items_by_id.values()
	}

	pub fn items_by_name(&self) -> impl Iterator<Item = &EquipableEntry> {
		self.itemids_by_name
			.iter()
			.map(|id| self.items_by_id.get(&id).unwrap())
	}

	pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Item> {
		self.items_by_id.get_mut(id).map(|entry| &mut entry.item)
	}

	pub fn set_equipped(&mut self, id: &Uuid, equipped: bool) {
		let Some(entry) = self.items_by_id.get_mut(&id) else { return; };
		entry.is_equipped = equipped;
	}
}

impl mutator::Container for Inventory {
	fn id(&self) -> Option<String> {
		Some("Inventory".into())
	}

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		for entry in self.items_by_name() {
			stats.apply_from(entry);
		}
	}
}
