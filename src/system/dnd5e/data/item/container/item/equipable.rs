use crate::kdl_ext::NodeContext;
use crate::system::{
	dnd5e::data::{
		character::Character,
		item::{container::item::AsItem, Item, Kind},
	},
	mutator,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::path::Path;

#[derive(Clone, PartialEq, Debug)]
pub struct EquipableEntry {
	pub id_path: Vec<uuid::Uuid>,
	pub item: Item,
	pub is_equipped: bool,
}

impl AsItem for EquipableEntry {
	fn from_item(item: Item) -> Self {
		Self {
			id_path: Vec::new(),
			item,
			is_equipped: false,
		}
	}

	fn set_id_path(&mut self, id: Vec<uuid::Uuid>) {
		self.item.set_id_path(id.clone());
		self.id_path = id;
	}

	fn id_path(&self) -> Option<&Vec<uuid::Uuid>> {
		Some(&self.id_path)
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

impl mutator::Group for EquipableEntry {
	type Target = Character;

	fn set_data_path(&self, parent: &std::path::Path) {
		let path_to_item = parent.join(&self.item.name);
		if let Kind::Equipment(equipment) = &self.item.kind {
			equipment.set_data_path(&path_to_item);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, parent: &Path) {
		let Kind::Equipment(equipment) = &self.item.kind else {
			return;
		};
		if !self.is_equipped {
			return;
		}

		let path_to_item = parent.join(&self.item.name);
		stats.apply_from(equipment, &path_to_item);
		if let Some(weapon) = &equipment.weapon {
			stats.add_feature(weapon.attack_action(self), &path_to_item);
		}
		if let Some(spell_container) = &self.item.spells {
			spell_container.add_spellcasting(stats, &self.id_path, &path_to_item);
		}
	}
}

impl FromKdl<NodeContext> for EquipableEntry {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let item = Item::from_kdl(node)?;
		let is_equipped = node.get_bool_opt("equipped")?.unwrap_or_default();
		Ok(Self {
			id_path: Vec::new(),
			is_equipped,
			item,
		})
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
