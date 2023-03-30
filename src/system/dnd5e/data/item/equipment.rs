use super::{armor::Armor, weapon::Weapon};
use crate::{
	kdl_ext::{FromKDL, NodeExt},
	system::dnd5e::{data::character::Character, BoxedCriteria, BoxedMutator},
	utility::MutatorGroup,
};
use std::path::Path;

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

	fn set_data_path(&self, path_to_item: &std::path::Path) {
		for mutator in &self.modifiers {
			mutator.set_data_path(path_to_item);
		}
		if let Some(armor) = &self.armor {
			armor.set_data_path(path_to_item);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, path_to_item: &Path) {
		for modifier in &self.modifiers {
			stats.apply(modifier, path_to_item);
		}
		if let Some(armor) = &self.armor {
			stats.apply_from(armor, path_to_item);
		}
		if let Some(shield) = &self.shield {
			stats
				.armor_class_mut()
				.push_bonus(*shield, path_to_item.to_owned());
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
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let criteria = match node.query("scope() > criteria")? {
			None => None,
			Some(entry_node) => {
				Some(ctx.parse_evaluator::<Character, Result<(), String>>(entry_node)?)
			}
		};

		// TODO: Item kdls current list these as `modifier`
		let mut mutators = Vec::new();
		for entry_node in node.query_all("scope() > mutator")? {
			mutators.push(ctx.parse_mutator(entry_node)?);
		}

		let armor = match node.query("scope() > armor")? {
			None => None,
			Some(node) => Some(Armor::from_kdl(node, &mut ctx.next_node())?),
		};
		let shield = match node.query("scope() > shield")? {
			None => None,
			Some(node) => Some(node.get_i64_req("bonus")? as i32),
		};
		let weapon = match node.query("scope() > weapon")? {
			None => None,
			Some(node) => Some(Weapon::from_kdl(node, &mut ctx.next_node())?),
		};
		let attunement = match node.query("scope() > attunement")? {
			None => None,
			Some(_node) => {
				None // TODO: Some(Attunement::from_kdl(node, &mut ctx.next_node())?)
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
