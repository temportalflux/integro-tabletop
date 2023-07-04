use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder},
	system::dnd5e::data::{
		character::Character,
		currency::Wallet,
		item::{Item, Restriction},
	},
	utility::MutatorGroup,
};
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

mod as_item;
pub use as_item::*;
mod equipable;
pub use equipable::*;
mod capacity;
pub use capacity::*;

pub type Inventory = Container<EquipableEntry>;

#[derive(Clone, PartialEq, Debug)]
pub struct Container<T> {
	pub capacity: Capacity,
	pub restriction: Option<Restriction>,
	items_by_id: HashMap<Uuid, T>,
	itemids_by_name: Vec<Uuid>,
	wallet: Wallet,
}

impl<T> Default for Container<T> {
	fn default() -> Self {
		Self {
			capacity: Default::default(),
			restriction: Default::default(),
			items_by_id: Default::default(),
			itemids_by_name: Default::default(),
			wallet: Default::default(),
		}
	}
}

impl<T> Container<T> {
	pub fn new() -> Self {
		Self {
			capacity: Capacity::default(),
			restriction: None,
			items_by_id: HashMap::new(),
			itemids_by_name: Vec::new(),
			wallet: Wallet::default(),
		}
	}

	pub fn wallet(&self) -> &Wallet {
		&self.wallet
	}

	pub fn wallet_mut(&mut self) -> &mut Wallet {
		&mut self.wallet
	}

	pub fn iter_by_name(&self) -> impl Iterator<Item = (&Uuid, &T)> {
		self.itemids_by_name
			.iter()
			.filter_map(|id| self.items_by_id.get(&id).map(|item| (id, item)))
	}
}

impl<T: AsItem> Container<T> {
	pub fn get_item(&self, id: &Uuid) -> Option<&Item> {
		self.items_by_id.get(id).map(|entry| entry.as_item())
	}

	pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Item> {
		self.items_by_id
			.get_mut(id)
			.map(|entry| entry.as_item_mut())
	}

	pub fn get_mut_at_path<'c>(&'c mut self, id_path: &Vec<Uuid>) -> Option<&'c mut Item> {
		let mut iter = id_path.iter();
		let Some(first_id) = iter.next() else { return None; };
		let Some(mut item) = self.get_mut(first_id) else { return None };
		for id in iter {
			let Some(container) = &mut item.items else { return None; };
			let Some(next_item) = container.get_mut(id) else { return None; };
			item = next_item;
		}
		Some(item)
	}

	fn push_entry(&mut self, entry: T) -> Uuid {
		let id = Uuid::new_v4();
		let search = self.itemids_by_name.binary_search_by(|id| {
			let Some(entry_item) = self.get_item(id) else {
				return std::cmp::Ordering::Less;
			};
			entry_item.name.cmp(&entry.as_item().name)
		});
		let idx = match search {
			// an item with the same name already exists at this index
			Ok(idx) => idx,
			// no item with the name exists, this is the index to insert to maintain sort-order
			Err(idx) => idx,
		};
		self.itemids_by_name.insert(idx, id.clone());
		self.items_by_id.insert(id.clone(), entry);
		id
	}

	pub fn push(&mut self, item: Item) -> Uuid {
		self.push_entry(T::from_item(item))
	}

	pub fn insert(&mut self, item: Item) -> Uuid {
		if item.can_stack() {
			for (id, entry) in &mut self.items_by_id {
				if entry.as_item().can_add_to_stack(&item) {
					entry.as_item_mut().add_to_stack(item);
					return id.clone();
				}
			}
		}
		self.push(item)
	}

	/// Attempts to insert the item into the specified item container.
	/// If no such item at the id exists OR that item is not an item container,
	/// the provided item is inserted into this inventory.
	pub fn insert_to(&mut self, item: Item, container_id: &Option<Vec<Uuid>>) -> Vec<Uuid> {
		if let Some(container_id) = container_id {
			if let Some(existing_item) = self.get_mut_at_path(container_id) {
				if let Some(container) = &mut existing_item.items {
					let id = container.insert(item);
					let mut full_path = container_id.clone();
					full_path.push(id);
					return full_path;
				}
			}
		}
		vec![self.insert(item)]
	}

	pub fn remove(&mut self, id: &Uuid) -> Option<Item> {
		if let Ok(idx) = self.itemids_by_name.binary_search(id) {
			self.itemids_by_name.remove(idx);
		}
		self.items_by_id.remove(id).map(|entry| entry.into_item())
	}

	pub fn remove_at_path(&mut self, id_path: &Vec<Uuid>) -> Option<Item> {
		let count = id_path.len();
		let mut iter = id_path
			.iter()
			.enumerate()
			.map(|(idx, id)| (id, idx == count - 1));
		let Some((first_id, single_entry)) = iter.next() else { return None; };
		if single_entry {
			return self.remove(first_id);
		}
		let Some(mut item) = self.get_mut(first_id) else { return None; };
		for (id, is_last) in iter {
			let Some(container) = &mut item.items else { return None; };
			if is_last {
				return container.remove(id);
			}
			let Some(next_item) = container.get_mut(id) else { return None; };
			item = next_item;
		}
		None
	}
}

impl<T: AsItem + FromKDL> FromKDL for Container<T> {
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let wallet = node
			.query_opt_t::<Wallet>("scope() > wallet")?
			.unwrap_or_default();

		let capacity = match node.query_opt("scope() > capacity")? {
			Some(node) => {
				let count = node.get_i64_opt("count")?.map(|v| v as usize);
				let weight = node.get_f64_opt("weight")?;
				let volume = node.get_f64_opt("volume")?;
				Capacity {
					count,
					weight,
					volume,
				}
			}
			None => Default::default(),
		};

		let restriction = match node.query_opt("scope() > restriction")? {
			Some(node) => {
				let tags = node.query_str_all("scope() > tag", 0)?;
				let tags = tags.into_iter().map(str::to_owned).collect::<Vec<_>>();
				Some(Restriction { tags })
			}
			None => None,
		};

		let mut inventory = Self {
			wallet,
			capacity,
			restriction,
			items_by_id: HashMap::new(),
			itemids_by_name: Vec::new(),
		};

		for mut node in &mut node.query_all("scope() > item")? {
			let item = T::from_kdl(&mut node)?;
			inventory.push_entry(item);
		}

		Ok(inventory)
	}
}

impl<T: AsKdl> AsKdl for Container<T> {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_child_opt_t("wallet", &self.wallet);

		node.push_child_opt({
			let mut node = NodeBuilder::default();
			if let Some(count) = &self.capacity.count {
				node.push_entry(("count", *count as i64));
			}
			if let Some(weight) = &self.capacity.weight {
				node.push_entry(("weight", *weight as i64));
			}
			if let Some(volume) = &self.capacity.volume {
				node.push_entry(("volume", *volume as i64));
			}
			node.build("capacity")
		});

		if let Some(restriction) = &self.restriction {
			let mut restriction_node = NodeBuilder::default();
			for tag in &restriction.tags {
				restriction_node.push_child_t("tag", tag);
			}
			node.push_child_opt(restriction_node.build("restriction"));
		}

		for id in &self.itemids_by_name {
			let Some(entry) = self.items_by_id.get(id) else { continue; };
			node.push_child(entry.as_kdl().build("item"));
		}

		node
	}
}

impl Inventory {
	pub fn is_equipped(&self, id: &Uuid) -> bool {
		self.items_by_id
			.get(id)
			.map(|entry| entry.is_equipped)
			.unwrap_or(false)
	}

	pub fn entries(&self) -> impl Iterator<Item = &EquipableEntry> {
		self.items_by_id.values()
	}

	pub fn set_equipped(&mut self, id: &Uuid, equipped: bool) {
		let Some(entry) = self.items_by_id.get_mut(&id) else { return; };
		entry.is_equipped = equipped;
	}
}

impl MutatorGroup for Inventory {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_self = parent.join("Inventory");
		for (_, entry) in self.iter_by_name() {
			entry.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		for (_id, entry) in self.iter_by_name() {
			stats.apply_from(entry, parent);
		}
	}
}
