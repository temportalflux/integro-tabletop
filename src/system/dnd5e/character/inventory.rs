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
	pub description: String,
	pub weight: u32,
	pub cost: u32,
	pub notes: String,
}
