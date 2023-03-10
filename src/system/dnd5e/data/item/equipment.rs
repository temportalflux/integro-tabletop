use super::{armor::Armor, weapon::Weapon};
use crate::{
	kdl_ext::{NodeQueryExt, ValueIdx},
	system::{
		core::NodeRegistry,
		dnd5e::{data::character::Character, BoxedCriteria, BoxedMutator, FromKDL},
	},
	utility::MutatorGroup,
};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Equipment {
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

impl MutatorGroup for Equipment {
	type Target = Character;

	fn apply_mutators<'c>(&self, stats: &mut Character) {
		for modifier in &self.modifiers {
			stats.apply(modifier);
		}
		if let Some(armor) = &self.armor {
			stats.apply_from(armor);
		}
		if let Some(shield) = &self.shield {
			let source = stats.source_path();
			stats.armor_class_mut().push_bonus(*shield, source);
		}
	}
}

impl Equipment {
	/// Returs Ok if the item can currently be equipped, otherwise returns a user-displayable reason why it cannot be equipped.
	pub fn can_be_equipped(&self, state: &Character) -> Result<(), String> {
		match &self.criteria {
			Some(criteria) => state.evaluate(criteria),
			None => Ok(()),
		}
	}
}

impl FromKDL for Equipment {
	fn from_kdl(
		node: &kdl::KdlNode,
		_value_idx: &mut ValueIdx,
		node_reg: &NodeRegistry,
	) -> anyhow::Result<Self> {
		let criteria = match node.query("criteria")? {
			None => None,
			Some(entry_node) => {
				Some(node_reg.parse_evaluator::<Character, Result<(), String>>(entry_node)?)
			}
		};

		// TODO: Item kdls current list these as `modifier`
		let mutators = {
			let mut mutators = Vec::new();
			for entry_node in node.query_all("mutator")? {
				mutators.push(node_reg.parse_mutator(entry_node)?);
			}
			mutators
		};

		let armor = match node.query("armor")? {
			None => None,
			Some(node) => Some(Armor::from_kdl(node, &mut ValueIdx::default(), node_reg)?),
		};
		let shield = match node.query("shield")? {
			None => None,
			Some(node) => Some(node.get_i64("bonus")? as i32),
		};
		let weapon = match node.query("weapon")? {
			None => None,
			Some(node) => Some(Weapon::from_kdl(node, &mut ValueIdx::default(), node_reg)?),
		};
		let attunement = match node.query("attunement")? {
			None => None,
			Some(_node) => {
				None // TODO: Some(Attunement::from_kdl(node, &mut ValueIdx::default(), system)?)
			}
		};

		Ok(Self {
			criteria,
			modifiers: mutators,
			armor,
			shield,
			weapon,
			attunement,
		})
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Attunement {
	pub modifiers: Vec<BoxedMutator>,
}
