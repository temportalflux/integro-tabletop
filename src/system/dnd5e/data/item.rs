use super::currency::Wallet;
use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeContext, NodeExt},
	system::{
		core::SourceId,
		dnd5e::{data::character::Character, mutator::AddAction, DnD5e, SystemComponent},
	},
	utility::{MutatorGroup, NotInList},
};
use std::{collections::HashMap, path::Path};
use uuid::Uuid;

pub mod armor;
pub mod equipment;
pub mod weapon;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Item {
	pub name: String,
	pub description: Option<String>,
	pub weight: f32,
	// TODO: When browsing items to add to inventory, there should be a PURCHASE option for buying
	// some quantity of an item and immediately removing the total from the characters's wallet.
	pub worth: Wallet,
	pub notes: Option<String>,
	pub kind: ItemKind,
	pub tags: Vec<String>,
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
		matches!(&self.kind, ItemKind::Simple { .. })
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
	type System = DnD5e;

	fn add_component(self, source_id: SourceId, system: &mut Self::System) {
		system.items.insert(source_id, self);
	}
}

impl FromKDL for Item {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let name = node.get_str_req("name")?.to_owned();
		let mut weight = node.get_f64_opt("weight")?.unwrap_or(0.0) as f32;
		let description = node
			.query_str_opt("scope() > description", 0)?
			.map(str::to_owned);

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
			name,
			description,
			weight,
			worth,
			notes,
			kind,
			tags,
		})
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

#[derive(Clone, PartialEq, Debug)]
pub struct EquipableEntry {
	pub id: Uuid,
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
			let mutator = AddAction(weapon.attack_action(self));
			stats.apply(&mutator.into(), &path_to_item);
		}
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Inventory {
	items_by_id: HashMap<Uuid, EquipableEntry>,
	itemids_by_name: Vec<Uuid>,
	wallet: Wallet,
}

impl Inventory {
	pub fn new() -> Self {
		Self {
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

	pub fn get_item(&self, id: &Uuid) -> Option<&Item> {
		self.items_by_id.get(id).map(|entry| &entry.item)
	}

	pub fn is_equipped(&self, id: &Uuid) -> bool {
		self.items_by_id
			.get(id)
			.map(|entry| entry.is_equipped)
			.unwrap_or(false)
	}

	pub fn insert(&mut self, item: Item) -> Uuid {
		if item.can_stack() {
			for (_id, entry) in &mut self.items_by_id {
				if entry.item.can_add_to_stack(&item) {
					entry.item.add_to_stack(item);
					return entry.id.clone();
				}
			}
		}

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
		id
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

impl MutatorGroup for Inventory {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_self = parent.join("Inventory");
		for entry in self.items_by_name() {
			entry.set_data_path(&path_to_self);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		for entry in self.items_by_name() {
			stats.apply_from(entry, parent);
		}
	}
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
				description: None,
				weight: 0.2,
				worth: Wallet::from([(1, currency::Kind::Copper)]),
				notes: None,
				kind: ItemKind::Simple { count: 5 },
				tags: vec![],
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
				description: None,
				weight: 65.0,
				worth: Wallet::from([(1500, currency::Kind::Gold)]),
				notes: None,
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
				tags: vec![],
			};
			assert_eq!(from_doc(doc)?, expected);
			Ok(())
		}
	}
}
