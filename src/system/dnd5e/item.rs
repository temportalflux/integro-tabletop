use crate::system::dnd5e::{character::Character, mutator};
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

pub struct ItemWithId<'a>(&'a Uuid, &'a Item);
impl<'a> ItemWithId<'a> {
	pub fn id(&self) -> &'a Uuid {
		self.0
	}

	pub fn item(&self) -> &'a Item {
		self.1
	}
}
impl<'a> mutator::Container for ItemWithId<'a> {
	fn id(&self) -> Option<String> {
		Some(self.item().name.clone())
	}

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		if let ItemKind::Equipment(equipment) = &self.item().kind {
			stats.apply_from(equipment);
			if equipment.is_equipped {
				if let Some(weapon) = &equipment.weapon {
					stats
						.actions_mut()
						.push(weapon.attack_action(self.item().name.clone(), self.id()));
				}
			}
		}
	}
}

#[derive(Clone, PartialEq, Default)]
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

	pub fn items<'a>(&'a self) -> Vec<ItemWithId<'a>> {
		self.itemids_by_name
			.iter()
			.map(|id| ItemWithId(id, self.items_by_id.get(&id).unwrap()))
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

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		for item_with_id in self.items() {
			stats.apply_from(&item_with_id);
		}
	}
}
