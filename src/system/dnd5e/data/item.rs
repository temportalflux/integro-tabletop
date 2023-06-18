use super::{currency::Wallet, description, Rarity};
use crate::{
	kdl_ext::{AsKdl, DocumentExt, FromKDL, NodeBuilder, NodeContext, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{data::character::Character, SystemComponent},
	},
	utility::{MutatorGroup, NotInList},
};
use std::{collections::HashMap, path::Path, str::FromStr};
use uuid::Uuid;

pub mod armor;
pub mod equipment;
pub mod weapon;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Item {
	pub id: SourceId,
	pub name: String,
	pub description: description::Info,
	pub rarity: Option<Rarity>,
	pub weight: f32,
	// TODO: When browsing items to add to inventory, there should be a PURCHASE option for buying
	// some quantity of an item and immediately removing the total from the characters's wallet.
	pub worth: Wallet,
	pub notes: Option<String>,
	pub kind: ItemKind,
	pub tags: Vec<String>,
	// TODO: Tests for item containers
	pub items: Option<Inventory<Item>>,
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

	pub fn can_stack(&self) -> bool {
		matches!(&self.kind, ItemKind::Simple { .. }) && self.items.is_none()
	}

	pub fn can_add_to_stack(&self, stackable: &Item) -> bool {
		assert!(stackable.can_stack());
		if !self.can_stack() {
			return false;
		}

		// There are 2 properties we do not check here.
		// `kind` is not checked because both items are `stackable` aka they are both simple items.
		// We don't care how many are in each simple item stack,
		// those will be combined if the other properties are equivalent.
		// `notes` is not checked because thats extra user data that is stack-agnostic.
		// If the user wants to have distinct stacks, they can rename the item.
		self.name == stackable.name
			&& self.description == stackable.description
			&& self.rarity == stackable.rarity
			&& self.weight == stackable.weight
			&& self.worth == stackable.worth
			&& self.tags == stackable.tags
	}

	pub fn add_to_stack(&mut self, other: Item) {
		assert!(self.can_stack());
		assert!(other.can_stack());
		match (&mut self.kind, other.kind) {
			(ItemKind::Simple { count: dst }, ItemKind::Simple { count: src }) => {
				*dst += src;
			}
			_ => {
				panic!("attempting to stack item with non-stackable item");
			}
		}
	}
}

crate::impl_kdl_node!(Item, "item");

impl SystemComponent for Item {
	fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"name": self.name.clone(),
		})
	}
}

impl FromKDL for Item {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		// TODO: Items can have empty ids if they are completely custom in the sheet
		let id = ctx.parse_source_opt(node)?.unwrap_or_default();

		let name = node.get_str_req("name")?.to_owned();
		let rarity = match node.query_str_opt("scope() > rarity", 0)? {
			Some(value) => Some(Rarity::from_str(value)?),
			None => None,
		};
		let mut weight = node.get_f64_opt("weight")?.unwrap_or(0.0) as f32;
		let description = match node.query_opt("scope() > description")? {
			None => description::Info::default(),
			Some(node) => description::Info::from_kdl(node, &mut ctx.next_node())?,
		};

		let worth = match node.query("scope() > worth")? {
			Some(node) => Wallet::from_kdl(node, &mut ctx.next_node())?,
			None => Wallet::default(),
		};

		let notes = node.query_str_opt("scope() > notes", 0)?.map(str::to_owned);
		let tags = {
			let mut tags = Vec::new();
			for tag in node.query_str_all("scope() > tag", 0)? {
				tags.push(tag.to_owned());
			}
			tags
		};
		let kind = match node.query("scope() > kind")? {
			Some(node) => ItemKind::from_kdl(node, &mut ctx.next_node())?,
			None => ItemKind::default(),
		};

		let items = match node.query_opt("scope() > items")? {
			None => None,
			Some(node) => Some(Inventory::<Item>::from_kdl(node, &mut ctx.next_node())?),
		};

		// Items are defined with the weight being representative of the stack,
		// but are used as the weight being representative of a single item
		// (total weight being calculated on the fly).
		// TODO: This will get wonky when saving/loading characters.
		// Use a special flag to indicate per-item vs per-stack weight?
		if weight > 0.0 {
			if let ItemKind::Simple { count } = &kind {
				weight /= *count as f32;
			}
		}

		Ok(Self {
			id,
			name,
			description,
			rarity,
			weight,
			worth,
			notes,
			kind,
			tags,
			items,
		})
	}
}
impl AsKdl for Item {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("name", self.name.clone()));

		node.push_child_opt_t("source", &self.id);
		if let Some(rarity) = &self.rarity {
			node.push_child_entry("rarity", rarity.to_string());
		}

		if self.weight > 0.0 {
			let mut stack_weight = self.weight as f64;
			if let ItemKind::Simple { count } = &self.kind {
				stack_weight *= *count as f64;
			}
			node.push_entry(("weight", stack_weight));
		}

		node.push_child_opt_t("description", &self.description);
		node.push_child_opt_t("worth", &self.worth);

		if let Some(notes) = &self.notes {
			node.push_child_t("notes", notes);
		}

		for tag in &self.tags {
			node.push_child_t("tag", tag);
		}

		if self.kind != ItemKind::default() {
			node.push_child_t("kind", &self.kind);
		}

		if let Some(items) = &self.items {
			node.push_child_t("items", items);
		}

		node
	}
}

#[derive(Clone, PartialEq, Debug)]
pub enum ItemKind {
	Simple { count: u32 },
	Equipment(equipment::Equipment),
}

impl Default for ItemKind {
	fn default() -> Self {
		Self::Simple { count: 1 }
	}
}

impl FromKDL for ItemKind {
	fn from_kdl(node: &kdl::KdlNode, ctx: &mut NodeContext) -> anyhow::Result<Self> {
		match node.get_str_req(ctx.consume_idx())? {
			"Simple" => {
				let count = node.get_i64_opt("count")?.unwrap_or(1) as u32;
				Ok(Self::Simple { count })
			}
			"Equipment" => {
				let equipment = equipment::Equipment::from_kdl(node, ctx)?;
				Ok(Self::Equipment(equipment))
			}
			value => Err(NotInList(value.into(), vec!["Simple", "Equipment"]).into()),
		}
	}
}
impl AsKdl for ItemKind {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		match self {
			Self::Simple { count } => {
				node.push_entry("Simple");
				if *count > 1 {
					node.push_entry(("count", *count as i64));
				}
			}
			Self::Equipment(equipment) => {
				node.push_entry("Equipment");
				node += equipment.as_kdl();
			}
		}
		node
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct EquipableEntry {
	pub item: Item,
	pub is_equipped: bool,
}
impl MutatorGroup for EquipableEntry {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_item = parent.join(&self.item.name);
		if let ItemKind::Equipment(equipment) = &self.item.kind {
			equipment.set_data_path(&path_to_item);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let ItemKind::Equipment(equipment) = &self.item.kind else { return; };
		if !self.is_equipped {
			return;
		}

		let path_to_item = parent.join(&self.item.name);
		stats.apply_from(equipment, &path_to_item);
		if let Some(weapon) = &equipment.weapon {
			stats.add_feature(&weapon.attack_action(self), &path_to_item);
		}
	}
}

pub trait AsItem {
	fn from_item(item: Item) -> Self;
	fn into_item(self) -> Item;
	fn as_item(&self) -> &Item;
	fn as_item_mut(&mut self) -> &mut Item;
}
impl AsItem for Item {
	fn from_item(item: Item) -> Self {
		item
	}

	fn into_item(self) -> Item {
		self
	}

	fn as_item(&self) -> &Item {
		self
	}

	fn as_item_mut(&mut self) -> &mut Item {
		self
	}
}
impl AsItem for EquipableEntry {
	fn from_item(item: Item) -> Self {
		Self {
			item,
			is_equipped: false,
		}
	}

	fn into_item(self) -> Item {
		self.item
	}

	fn as_item(&self) -> &Item {
		&self.item
	}

	fn as_item_mut(&mut self) -> &mut Item {
		&mut self.item
	}
}

#[derive(Clone, PartialEq, Debug)]
pub struct Inventory<T> {
	pub capacity: ItemContainerCapacity,
	pub restriction: Option<Restriction>,
	items_by_id: HashMap<Uuid, T>,
	itemids_by_name: Vec<Uuid>,
	wallet: Wallet,
}
impl<T> Default for Inventory<T> {
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
impl<T> Inventory<T> {
	pub fn new() -> Self {
		Self {
			capacity: ItemContainerCapacity::default(),
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

impl<T: AsItem> Inventory<T> {
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

impl Inventory<EquipableEntry> {
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

impl MutatorGroup for Inventory<EquipableEntry> {
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

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ItemContainerCapacity {
	pub count: Option<usize>,
	// Unit: pounds (lbs)
	pub weight: Option<f64>,
	// Unit: cubic feet
	pub volume: Option<f64>,
}

impl<T: AsItem + FromKDL> FromKDL for Inventory<T> {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let wallet = match node.query_opt("scope() > wallet")? {
			Some(node) => Wallet::from_kdl(node, &mut ctx.next_node())?,
			None => Default::default(),
		};

		let capacity = match node.query_opt("scope() > capacity")? {
			Some(node) => {
				let count = node.get_i64_opt("count")?.map(|v| v as usize);
				let weight = node.get_f64_opt("weight")?;
				let volume = node.get_f64_opt("volume")?;
				ItemContainerCapacity {
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

		for node in node.query_all("scope() > item")? {
			let item = T::from_kdl(node, &mut ctx.next_node())?;
			inventory.push_entry(item);
		}

		Ok(inventory)
	}
}
impl<T: AsKdl> AsKdl for Inventory<T> {
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

impl FromKDL for EquipableEntry {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let item = Item::from_kdl(node, ctx)?;
		let is_equipped = node.get_bool_opt("equipped")?.unwrap_or_default();
		Ok(Self { is_equipped, item })
	}
}
impl AsKdl for EquipableEntry {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = self.item.as_kdl();
		if self.is_equipped {
			node.push_entry(("equipped", true));
		}
		node
	}
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Restriction {
	pub tags: Vec<String>,
}

#[cfg(test)]
mod test {
	use super::*;

	mod item {
		use super::*;
		use crate::{
			kdl_ext::NodeContext,
			system::{
				core::NodeRegistry,
				dnd5e::{
					data::{
						currency,
						item::{armor::Armor, equipment::Equipment},
						roll::Modifier,
						ArmorClassFormula, Skill,
					},
					mutator::{AddModifier, ModifierKind},
				},
			},
			utility::Selector,
		};

		fn from_doc(doc: &str) -> anyhow::Result<Item> {
			let mut ctx = NodeContext::registry(NodeRegistry::default_with_mut::<AddModifier>());
			let document = doc.parse::<kdl::KdlDocument>()?;
			let node = document
				.query("scope() > item")?
				.expect("missing item node");
			Item::from_kdl(node, &mut ctx)
		}

		#[test]
		fn simple() -> anyhow::Result<()> {
			let doc = "item name=\"Torch\" weight=1.0 {
				worth 1 (Currency)\"Copper\"
				kind \"Simple\" count=5
			}";
			let expected = Item {
				name: "Torch".into(),
				weight: 0.2,
				worth: Wallet::from([(1, currency::Kind::Copper)]),
				kind: ItemKind::Simple { count: 5 },
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}

		#[test]
		fn equipment() -> anyhow::Result<()> {
			let doc = "item name=\"Plate Armor\" weight=65.0 {
				worth 1500 (Currency)\"Gold\"
				kind \"Equipment\" {
					armor \"Heavy\" {
						formula base=18
						min-strength 15
					}
					mutator \"add_modifier\" \"Disadvantage\" (Skill)\"Specific\" \"Stealth\"
				}
			}";
			let expected = Item {
				name: "Plate Armor".into(),
				weight: 65.0,
				worth: Wallet::from([(1500, currency::Kind::Gold)]),
				kind: ItemKind::Equipment(Equipment {
					criteria: None,
					mutators: vec![AddModifier {
						modifier: Modifier::Disadvantage,
						context: None,
						kind: ModifierKind::Skill(Selector::Specific(Skill::Stealth)),
					}
					.into()],
					armor: Some(Armor {
						kind: armor::Kind::Heavy,
						formula: ArmorClassFormula {
							base: 18,
							bonuses: vec![],
						},
						min_strength_score: Some(15),
					}),
					shield: None,
					weapon: None,
					attunement: None,
				}),
				..Default::default()
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
