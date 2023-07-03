use super::Weapon;
use crate::{
	kdl_ext::{DocumentQueryExt, NodeBuilder, ValueExt},
	system::dnd5e::data::{
		action::{Action, AttackCheckKind, AttackKind},
		item::weapon,
		Ability,
	},
};
use enumset::EnumSet;
use itertools::Itertools;
use std::{collections::HashSet, str::FromStr};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Restriction {
	pub weapon_kind: EnumSet<weapon::Kind>,
	pub attack_kind: EnumSet<AttackKind>,
	pub ability: HashSet<Ability>,
	pub properties: Vec<(weapon::Property, bool)>,
}

impl std::fmt::Display for Restriction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut entries = Vec::new();

		if !self.weapon_kind.is_empty() {
			let desc = if self.weapon_kind == EnumSet::all() {
				let weapon_kinds = self
					.weapon_kind
					.iter()
					.sorted()
					.map(|kind| kind.to_string())
					.collect::<Vec<_>>();
				let desc = crate::utility::list_as_english(weapon_kinds, "or").unwrap_or_default();
				format!("is a {desc} weapon")
			} else {
				format!("is a weapon")
			};
			entries.push(desc);
		}

		if !self.attack_kind.is_empty() && self.attack_kind != EnumSet::all() {
			let attack_kinds = self
				.attack_kind
				.iter()
				.sorted()
				.map(|kind| kind.to_string())
				.collect::<Vec<_>>();
			let attack_kinds = crate::utility::list_as_english(attack_kinds, "or");
			entries.push(format!("is a {} attack", attack_kinds.unwrap_or_default()));
		}

		let abilities = self
			.ability
			.iter()
			.sorted()
			.map(Ability::long_name)
			.map(str::to_owned)
			.collect::<Vec<_>>();
		let abilities = crate::utility::list_as_english(abilities, "or");
		if let Some(desc) = abilities {
			entries.push(format!("uses the {desc} ability"));
		}

		for (property, required_or_barred) in &self.properties {
			if *required_or_barred {
				entries.push(format!("has the {} property", property.display_name()));
			} else {
				entries.push(format!(
					"does not have the {} property",
					property.display_name()
				));
			}
		}

		write!(
			f,
			"{}",
			crate::utility::list_as_english(entries, "and").unwrap_or_default()
		)
	}
}

impl Restriction {
	pub fn does_weapon_meet(&self, weapon: &Weapon) -> bool {
		// the action must be an attack which is one of the provided weapon kinds
		if !self.weapon_kind.is_empty() {
			if !self.weapon_kind.contains(weapon.kind) {
				return false;
			}
		}
		// the action must be an attack which is one of the provided attack kinds
		if !self.attack_kind.is_empty() {
			if !self.attack_kind.contains(weapon.attack_kind()) {
				return false;
			}
		}
		// the action must be an attack which uses one of the provided abilities
		if !self.ability.is_empty() {
			if !self.ability.contains(&weapon.attack_ability()) {
				return false;
			}
		}
		// the action must be an attack which has or doesn't have specific weapon properties
		if !self.properties.is_empty() {
			for (property, required_else_barred) in &self.properties {
				let has_property = weapon.properties.contains(property);
				if has_property != *required_else_barred {
					return false;
				}
			}
		}
		true
	}

	pub fn does_action_meet(&self, action: &Action) -> bool {
		// the action must be an attack which is one of the provided weapon kinds
		if !self.weapon_kind.is_empty() {
			let Some(attack) = &action.attack else {
				return false;
			};
			let Some(kind) = &attack.weapon_kind else {
				return false;
			};
			if !self.weapon_kind.contains(*kind) {
				return false;
			}
		}
		// the action must be an attack which is one of the provided attack kinds
		if !self.attack_kind.is_empty() {
			let Some(attack) = &action.attack else { return false; };
			let Some(atk_kind) = &attack.kind else { return false; };
			if !self.attack_kind.contains(atk_kind.kind()) {
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
	fn from_kdl_reader<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let weapon_kind = match node.query_opt("scope() > weapon")? {
			None => EnumSet::empty(),
			Some(node) => {
				let mut allowed_kinds = EnumSet::empty();
				for entry in node.entries() {
					allowed_kinds.insert(weapon::Kind::from_str(entry.value().as_str_req()?)?);
				}
				allowed_kinds
			}
		};
		let attack_kind = match node.query_opt("scope() > attack")? {
			None => EnumSet::empty(),
			Some(node) => {
				let mut allowed_kinds = EnumSet::empty();
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
		for mut node in node.query_all("scope() > property")? {
			let property = weapon::Property::from_kdl_reader(&mut node)?;
			let required = node.next_bool_req()?;
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
