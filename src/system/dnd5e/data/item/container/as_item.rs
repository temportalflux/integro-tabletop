use crate::system::dnd5e::data::item::Item;

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
