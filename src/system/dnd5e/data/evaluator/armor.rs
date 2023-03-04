use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt},
	system::dnd5e::{
		data::{
			character::Character,
			item::{armor, EquipableEntry, ItemKind},
		},
		DnD5e, FromKDL, KDLNode,
	},
};
use std::{collections::HashSet, str::FromStr};

/// Checks if the character has armor equipped.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct HasArmorEquipped {
	/// By default, this criteria checks if a piece of armor is equipped.
	/// If this flag is true, the criteria checks if no armor is equipped (or no armor of a particular set of types).
	pub inverted: bool,
	/// The list of armor types to check. If empty, all armor is considered.
	pub kinds: HashSet<armor::Kind>,
}
impl HasArmorEquipped {
	fn kind_list(&self, joiner: &str) -> Option<String> {
		if self.kinds.is_empty() {
			return None;
		}
		let mut sorted_kinds = self.kinds.iter().collect::<Vec<_>>();
		sorted_kinds.sort();
		let mut kinds = sorted_kinds
			.into_iter()
			.map(|kind| format!("{kind:?}").to_lowercase())
			.collect::<Vec<_>>();
		Some(match kinds.len() {
			0 => unimplemented!(),
			1 => kinds.into_iter().next().unwrap(),
			2 => kinds.join(format!(" {joiner} ").as_str()),
			_ => {
				if let Some(last) = kinds.last_mut() {
					*last = format!("{joiner} {last}");
				}
				kinds.join(", ")
			}
		})
	}
}

impl crate::utility::Evaluator for HasArmorEquipped {
	type Context = Character;
	type Item = Result<(), String>;

	fn evaluate(&self, character: &Self::Context) -> Result<(), String> {
		for EquipableEntry {
			id: _,
			item,
			is_equipped,
		} in character.inventory().entries()
		{
			if !item.is_equipable() || !is_equipped {
				continue;
			}
			let ItemKind::Equipment(equipment) = &item.kind else { continue; };
			let Some(armor) = &equipment.armor else { continue; };
			if self.kinds.is_empty() || self.kinds.contains(&armor.kind) {
				return match self.inverted {
					false => Ok(()),
					true => Err(format!("\"{}\" is already equipped.", item.name)),
				};
			}
		}
		match self.inverted {
			false => Err(format!(
				"No {}armor equipped",
				match self.kind_list("or") {
					None => String::new(),
					Some(kind_list) => format!("{kind_list} "),
				}
			)),
			true => Ok(()),
		}
	}
}

impl KDLNode for HasArmorEquipped {
	fn id() -> &'static str {
		"has_armor_equipped"
	}
}

impl FromKDL for HasArmorEquipped {
	type System = DnD5e;

	fn from_kdl(node: &kdl::KdlNode, _system: &Self::System) -> anyhow::Result<Self> {
		let inverted = node.get_bool_opt("inverted")?.unwrap_or_default();
		let mut kinds = HashSet::new();
		if let Some(children) = node.children() {
			for kind_str_result in children.query_str_all("kind", 0)? {
				kinds.insert(armor::Kind::from_str(kind_str_result?)?);
			}
		}
		Ok(Self { inverted, kinds })
	}
}
