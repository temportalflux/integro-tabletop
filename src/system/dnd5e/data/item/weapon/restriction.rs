use itertools::Itertools;

use crate::{
	kdl_ext::{DocumentExt, NodeBuilder, NodeExt, ValueExt},
	system::dnd5e::data::{
		action::{Action, AttackCheckKind, AttackKind},
		item::weapon,
		Ability,
	},
};
use std::{collections::HashSet, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Restriction {
	pub weapon_kind: HashSet<weapon::Kind>,
	pub attack_kind: HashSet<AttackKind>,
	pub ability: HashSet<Ability>,
	pub properties: Vec<(weapon::Property, bool)>,
}

impl Restriction {
	pub fn does_action_meet(&self, action: &Action) -> bool {
		// the action must be an attack which is one of the provided weapon kinds
		if !self.weapon_kind.is_empty() {
			let Some(attack) = &action.attack else {
				return false;
			};
			let Some(kind) = &attack.weapon_kind else {
				return false;
			};
			if !self.weapon_kind.contains(kind) {
				return false;
			}
		}
		// the action must be an attack which is one of the provided attack kinds
		if !self.attack_kind.is_empty() {
			let Some(attack) = &action.attack else { return false; };
			let Some(atk_kind) = &attack.kind else { return false; };
			if !self.attack_kind.contains(&atk_kind.kind()) {
				return false;
			}
		}
		// the action must be an attack which uses one of the provided abilities
		if !self.ability.is_empty() {
			let Some(attack) = &action.attack else { return false; };
			let AttackCheckKind::AttackRoll { ability: atk_roll_ability, .. } = &attack.check else { return false; };
			if !self.ability.contains(atk_roll_ability) {
				return false;
			}
		}
		// the action must be an attack which has or doesn't have specific weapon properties
		if !self.properties.is_empty() {
			let Some(attack) = &action.attack else { return false; };
			for (property, required_else_barred) in &self.properties {
				let has_property = attack.properties.contains(property);
				if has_property != *required_else_barred {
					return false;
				}
			}
		}
		true
	}
}

impl crate::kdl_ext::FromKDL for Restriction {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let weapon_kind = match node.query_opt("scope() > weapon")? {
			None => HashSet::default(),
			Some(node) => {
				let mut allowed_kinds = HashSet::default();
				for entry in node.entries() {
					allowed_kinds.insert(weapon::Kind::from_str(entry.value().as_str_req()?)?);
				}
				allowed_kinds
			}
		};
		let attack_kind = match node.query_opt("scope() > attack")? {
			None => HashSet::default(),
			Some(node) => {
				let mut allowed_kinds = HashSet::default();
				for entry in node.entries() {
					allowed_kinds.insert(AttackKind::from_str(entry.value().as_str_req()?)?);
				}
				allowed_kinds
			}
		};
		let ability = match node.query_opt("scope() > ability")? {
			None => HashSet::default(),
			Some(node) => {
				let mut allowed_kinds = HashSet::default();
				for entry in node.entries() {
					allowed_kinds.insert(Ability::from_str(entry.value().as_str_req()?)?);
				}
				allowed_kinds
			}
		};

		let mut properties = Vec::new();
		for node in node.query_all("scope() > property")? {
			let mut ctx = ctx.next_node();
			let property = weapon::Property::from_kdl(node, &mut ctx)?;
			let required = node.get_bool_req(ctx.consume_idx())?;
			properties.push((property, required));
		}

		Ok(Self {
			weapon_kind,
			attack_kind,
			ability,
			properties,
		})
	}
}

impl crate::kdl_ext::AsKdl for Restriction {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if !self.weapon_kind.is_empty() {
			let kinds = self.weapon_kind.iter().sorted();
			node.push_child({
				let mut node = NodeBuilder::default();
				for kind in kinds {
					node.push_entry(kind.to_string());
				}
				node.build("weapon")
			});
		}
		if !self.attack_kind.is_empty() {
			let kinds = self.attack_kind.iter().sorted();
			node.push_child({
				let mut node = NodeBuilder::default();
				for kind in kinds {
					node.push_entry(kind.to_string());
				}
				node.build("attack")
			});
		}
		if !self.ability.is_empty() {
			let abilities = self.ability.iter().sorted();
			node.push_child({
				let mut node = NodeBuilder::default();
				for ability in abilities {
					node.push_entry(ability.to_string());
				}
				node.build("ability")
			});
		}

		for (property, required) in &self.properties {
			node.push_child(property.as_kdl().with_entry(*required).build("property"));
		}

		node
	}
}
