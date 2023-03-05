use crate::{
	kdl_ext::{DocumentQueryExt, NodeQueryExt, ValueIdx},
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

impl crate::utility::TraitEq for HasArmorEquipped {
	fn equals_trait(&self, other: &dyn crate::utility::TraitEq) -> bool {
		crate::utility::downcast_trait_eq(self, other)
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
					true => Err(format!("\"{}\" is equipped.", item.name)),
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

impl FromKDL<DnD5e> for HasArmorEquipped {
	fn from_kdl(node: &kdl::KdlNode, _: &mut ValueIdx, _system: &DnD5e) -> anyhow::Result<Self> {
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

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		system::dnd5e::data::{
			character::Persistent,
			item::{armor::Armor, equipment::Equipment, Item},
		},
		utility::{Evaluator, GenericEvaluator},
	};

	fn from_doc(doc: &str) -> anyhow::Result<GenericEvaluator<Character, Result<(), String>>> {
		DnD5e::defaulteval_parse_kdl::<HasArmorEquipped>(doc)
	}

	mod from_kdl {
		use super::*;

		#[test]
		fn simple() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_armor_equipped\"";
			let expected = HasArmorEquipped::default();
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}

		#[test]
		fn inverted() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_armor_equipped\" inverted=true";
			let expected = HasArmorEquipped {
				inverted: true,
				..Default::default()
			};
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}

		#[test]
		fn with_kinds() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_armor_equipped\" {
				kind \"Light\"
			}";
			let expected = HasArmorEquipped {
				kinds: [armor::Kind::Light].into(),
				..Default::default()
			};
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}

		#[test]
		fn with_not_kinds() -> anyhow::Result<()> {
			let doc_str = "evaluator \"has_armor_equipped\" inverted=true {
				kind \"Heavy\"
			}";
			let expected = HasArmorEquipped {
				inverted: true,
				kinds: [armor::Kind::Heavy].into(),
				..Default::default()
			};
			assert_eq!(from_doc(doc_str)?, expected.into());
			Ok(())
		}
	}

	mod kind_list {
		use super::*;

		#[test]
		fn to_kindlist_0() {
			assert_eq!(HasArmorEquipped::default().kind_list("and"), None);
		}

		#[test]
		fn to_kindlist_1() {
			assert_eq!(
				HasArmorEquipped {
					kinds: [armor::Kind::Medium].into(),
					..Default::default()
				}
				.kind_list("and"),
				Some("medium".into())
			);
		}

		#[test]
		fn to_kindlist_2() {
			assert_eq!(
				HasArmorEquipped {
					kinds: [armor::Kind::Light, armor::Kind::Medium].into(),
					..Default::default()
				}
				.kind_list("and"),
				Some("light and medium".into())
			);
		}

		#[test]
		fn to_kindlist_3plus() {
			assert_eq!(
				HasArmorEquipped {
					kinds: [armor::Kind::Light, armor::Kind::Medium, armor::Kind::Heavy].into(),
					..Default::default()
				}
				.kind_list("and"),
				Some("light, medium, and heavy".into())
			);
		}
	}

	mod evaluate {
		use super::*;

		fn character_with_armor(kinds: &[(armor::Kind, bool)]) -> Character {
			let mut persistent = Persistent::default();
			for (kind, equipped) in kinds {
				let id = persistent.inventory.insert(Item {
					name: format!("Armor{}", kind.to_string()),
					kind: ItemKind::Equipment(Equipment {
						armor: Some(Armor {
							kind: *kind,
							formula: Default::default(),
							min_strength_score: None,
						}),
						..Default::default()
					}),
					..Default::default()
				});
				persistent.inventory.set_equipped(&id, *equipped);
			}
			Character::from(persistent)
		}

		mod any {
			use super::*;

			#[test]
			fn no_equipment() {
				let evaluator = HasArmorEquipped::default();
				let character = character_with_armor(&[]);
				assert_eq!(
					evaluator.evaluate(&character),
					Err("No armor equipped".into())
				);
			}

			#[test]
			fn unequipped() {
				let evaluator = HasArmorEquipped::default();
				let with_medium = character_with_armor(&[(armor::Kind::Medium, false)]);
				assert_eq!(
					evaluator.evaluate(&with_medium),
					Err("No armor equipped".into())
				);
			}

			#[test]
			fn equipped() {
				let evaluator = HasArmorEquipped::default();
				let with_light = character_with_armor(&[(armor::Kind::Light, true)]);
				let with_medium = character_with_armor(&[(armor::Kind::Medium, true)]);
				let with_heavy = character_with_armor(&[(armor::Kind::Heavy, true)]);
				assert_eq!(evaluator.evaluate(&with_light), Ok(()));
				assert_eq!(evaluator.evaluate(&with_medium), Ok(()));
				assert_eq!(evaluator.evaluate(&with_heavy), Ok(()));
			}
		}

		mod single {
			use super::*;

			#[test]
			fn no_equipment() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[]);
				assert_eq!(
					evaluator.evaluate(&with_light),
					Err("No light armor equipped".into())
				);
			}

			#[test]
			fn unequipped() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[(armor::Kind::Light, false)]);
				assert_eq!(
					evaluator.evaluate(&with_light),
					Err("No light armor equipped".into())
				);
			}

			#[test]
			fn wrong() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[(armor::Kind::Heavy, true)]);
				assert_eq!(
					evaluator.evaluate(&with_light),
					Err("No light armor equipped".into())
				);
			}

			#[test]
			fn equipped() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[(armor::Kind::Light, true)]);
				assert_eq!(evaluator.evaluate(&with_light), Ok(()));
			}
		}

		mod multiple {
			use super::*;

			#[test]
			fn no_equipment() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light, armor::Kind::Medium].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[]);
				assert_eq!(
					evaluator.evaluate(&with_light),
					Err("No light or medium armor equipped".into())
				);
			}

			#[test]
			fn unequipped() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light, armor::Kind::Medium].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[(armor::Kind::Medium, false)]);
				assert_eq!(
					evaluator.evaluate(&with_light),
					Err("No light or medium armor equipped".into())
				);
			}

			#[test]
			fn wrong() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light, armor::Kind::Medium].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[(armor::Kind::Heavy, true)]);
				assert_eq!(
					evaluator.evaluate(&with_light),
					Err("No light or medium armor equipped".into())
				);
			}

			#[test]
			fn equipped() {
				let evaluator = HasArmorEquipped {
					kinds: [armor::Kind::Light, armor::Kind::Medium].into(),
					..Default::default()
				};
				let with_light = character_with_armor(&[(armor::Kind::Medium, true)]);
				assert_eq!(evaluator.evaluate(&with_light), Ok(()));
			}
		}

		mod none_allowed {
			use super::*;

			#[test]
			fn no_equipment() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					..Default::default()
				};
				let character = character_with_armor(&[]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}

			#[test]
			fn unequipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Heavy, false)]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}

			#[test]
			fn equipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Heavy, true)]);
				assert_eq!(
					evaluator.evaluate(&character),
					Err("\"ArmorHeavy\" is equipped.".into())
				);
			}
		}

		mod no_single {
			use super::*;

			#[test]
			fn no_equipment() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}

			#[test]
			fn unequipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Heavy, false)]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}

			#[test]
			fn equipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Heavy, true)]);
				assert_eq!(
					evaluator.evaluate(&character),
					Err("\"ArmorHeavy\" is equipped.".into())
				);
			}

			#[test]
			fn otherequipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Medium, true)]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}
		}

		mod no_multiple {
			use super::*;

			#[test]
			fn no_equipment() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Medium, armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}

			#[test]
			fn unequipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Medium, armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[
					(armor::Kind::Heavy, false),
					(armor::Kind::Medium, false),
				]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}

			#[test]
			fn equipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Medium, armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Medium, true)]);
				assert_eq!(
					evaluator.evaluate(&character),
					Err("\"ArmorMedium\" is equipped.".into())
				);
			}

			#[test]
			fn otherequipped() {
				let evaluator = HasArmorEquipped {
					inverted: true,
					kinds: [armor::Kind::Medium, armor::Kind::Heavy].into(),
					..Default::default()
				};
				let character = character_with_armor(&[(armor::Kind::Light, true)]);
				assert_eq!(evaluator.evaluate(&character), Ok(()));
			}
		}
	}
}
