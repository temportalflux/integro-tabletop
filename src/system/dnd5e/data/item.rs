use super::character::IndirectItem;
use crate::kdl_ext::NodeContext;
use crate::system::{
	dnd5e::data::{character::Character, currency::Wallet, description, Rarity},
	Block, SourceId,
};
use kdlize::OmitIfEmpty;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};
use std::{collections::HashMap, str::FromStr};

mod kind;
pub use kind::*;
pub mod armor;
pub mod container;
pub mod equipment;
pub mod restriction;
pub use restriction::Restriction;
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
	pub kind: Kind,
	pub tags: Vec<String>,
	// TODO: Tests for item containers
	// Items which are contained within this item instance.
	pub items: Option<container::ItemContainer<Item>>,
	// Items (specific by SourceId or custom defn) which should be converted into the item container when added to a character sheet.
	// The owning item must have a container (typically empty) if using this property.
	pub item_refs: Vec<IndirectItem>,
	pub spells: Option<container::SpellContainer>,
}

kdlize::impl_kdl_node!(Item, "item");

impl Item {
	/// Returns true if the item has the capability to be equipped (i.e. it is a piece of equipment).
	pub fn is_equipable(&self) -> bool {
		match &self.kind {
			Kind::Equipment(_) => true,
			_ => false,
		}
	}

	/// Returs Ok if the item can currently be equipped, otherwise returns a user-displayable reason why it cannot be equipped.
	pub fn can_be_equipped(&self, state: &Character) -> Result<(), String> {
		match &self.kind {
			Kind::Equipment(equipment) => equipment.can_be_equipped(state),
			_ => Ok(()),
		}
	}

	pub fn quantity(&self) -> u32 {
		match &self.kind {
			Kind::Simple { count } => *count,
			_ => 1,
		}
	}

	pub fn can_stack(&self) -> bool {
		matches!(&self.kind, Kind::Simple { .. }) && self.items.is_none()
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
			(Kind::Simple { count: dst }, Kind::Simple { count: src }) => {
				*dst += src;
			}
			_ => {
				panic!("attempting to stack item with non-stackable item");
			}
		}
	}

	pub fn create_stack(mut self, quantity: usize) -> Vec<Self> {
		let mut stack = Vec::with_capacity(1);
		let single = match (quantity, &mut self.kind) {
			(1, _) => Some(self),
			(n, Kind::Simple { count }) => {
				*count = n as u32;
				Some(self)
			}
			(n, _) => {
				stack.reserve(n - 1);
				stack.fill(self);
				None
			}
		};
		if let Some(single) = single {
			stack.push(single);
		}
		stack
	}
}

impl Block for Item {
	fn to_metadata(self) -> serde_json::Value {
		let mut contents: HashMap<&'static str, serde_json::Value> =
			[("name", self.name.into()), ("tags", self.tags.into())].into();
		if let Kind::Equipment(equipment) = self.kind {
			contents.insert("equipment", equipment.to_metadata());
		}
		if let Some(rarity) = &self.rarity {
			contents.insert("rarity", rarity.to_string().into());
		}
		serde_json::json!(contents)
	}
}

impl FromKdl<NodeContext> for Item {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		// TODO: Items can have empty ids if they are completely custom in the sheet
		let id = crate::kdl_ext::query_source_opt(node)?.unwrap_or_default();

		let name = node.get_str_req("name")?.to_owned();
		let rarity = match node.query_str_opt("scope() > rarity", 0)? {
			Some(value) => Some(Rarity::from_str(value)?),
			None => None,
		};
		let mut weight = node.get_f64_opt("weight")?.unwrap_or(0.0) as f32;
		let description = match node.query_opt("scope() > description")? {
			None => description::Info::default(),
			Some(mut node) => description::Info::from_kdl(&mut node)?,
		};

		let worth = node.query_opt_t::<Wallet>("scope() > worth")?.unwrap_or_default();

		let notes = node.query_str_opt("scope() > notes", 0)?.map(str::to_owned);
		let tags = {
			let mut tags = Vec::new();
			for tag in node.query_str_all("scope() > tag", 0)? {
				tags.push(tag.to_owned());
			}
			tags
		};
		let kind = node.query_opt_t::<Kind>("scope() > kind")?.unwrap_or_default();
		let items = node.query_opt_t::<container::ItemContainer<Item>>("scope() > items")?;
		let item_refs = match node.query_opt("scope() > items > templates")? {
			None => Vec::new(),
			Some(node) => node.query_all_t::<IndirectItem>("scope() > item")?,
		};
		let spells = node.query_opt_t("scope() > spells")?;

		// Items are defined with the weight being representative of the stack,
		// but are used as the weight being representative of a single item
		// (total weight being calculated on the fly).
		// TODO: This will get wonky when saving/loading characters.
		// Use a special flag to indicate per-item vs per-stack weight?
		if weight > 0.0 {
			if let Kind::Simple { count } = &kind {
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
			item_refs,
			spells,
		})
	}
}
impl AsKdl for Item {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(("name", self.name.clone()));
		node.push_child_t(("source", &self.id, OmitIfEmpty));
		node.push_child_t(("rarity", &self.rarity.as_ref().map(Rarity::to_string)));

		if self.weight > 0.0 {
			let mut stack_weight = self.weight as f64;
			if let Kind::Simple { count } = &self.kind {
				stack_weight *= *count as f64;
			}
			let stack_weight = (stack_weight * 1000.0).round() / 1000.0;
			node.push_entry(("weight", stack_weight));
		}

		node.push_child_t(("worth", &self.worth, OmitIfEmpty));
		node.push_children_t(("tag", self.tags.iter()));
		node.push_child_t(("description", &self.description, OmitIfEmpty));
		node.push_child_t(("notes", &self.notes));

		if self.kind != Kind::default() {
			node.push_child_t(("kind", &self.kind));
		}

		if let Some(items) = &self.items {
			let templates = {
				let mut node = NodeBuilder::default();
				node.push_children_t(("item", self.item_refs.iter()));
				node.build("templates")
			};
			node.push_child({
				let mut node = items.as_kdl();
				node.push_child((templates, OmitIfEmpty));
				node.build("items")
			});
		}

		node.push_child_t(("spells", &self.spells));

		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod item {
		use super::*;
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				dnd5e::{
					data::{
						currency,
						item::{armor::Armor, equipment::Equipment},
						roll::Modifier,
						ArmorClassFormula, Skill,
					},
					mutator::{AddModifier, ModifierKind},
				},
				generics,
			},
			utility::selector,
		};

		static NODE_NAME: &str = "item";

		fn node_ctx() -> NodeContext {
			NodeContext::registry(generics::Registry::default_with_mut::<AddModifier>())
		}

		#[test]
		fn simple() -> anyhow::Result<()> {
			let doc = "
				|item name=\"Torch\" weight=1.0 {
				|    worth 1 (Currency)\"Copper\"
				|    kind \"Simple\" count=5
				|}
			";
			let data = Item {
				name: "Torch".into(),
				weight: 0.2,
				worth: Wallet::from([(1, currency::Kind::Copper)]),
				kind: Kind::Simple { count: 5 },
				..Default::default()
			};
			assert_eq_fromkdl!(Item, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn equipment() -> anyhow::Result<()> {
			let doc = "
				|item name=\"Plate Armor\" weight=65.0 {
				|    worth 1500 (Currency)\"Gold\"
				|    kind \"Equipment\" {
				|        armor \"Heavy\" {
				|            formula base=18
				|            min-strength 15
				|        }
				|        mutator \"add_modifier\" \"Disadvantage\" (Skill)\"Specific\" \"Stealth\"
				|    }
				|}
			";
			let data = Item {
				name: "Plate Armor".into(),
				weight: 65.0,
				worth: Wallet::from([(1500, currency::Kind::Gold)]),
				kind: Kind::Equipment(Equipment {
					mutators: vec![AddModifier {
						modifier: Modifier::Disadvantage,
						context: None,
						kind: ModifierKind::Skill(selector::Value::Specific(Skill::Stealth)),
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
					..Default::default()
				}),
				..Default::default()
			};
			assert_eq_fromkdl!(Item, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
