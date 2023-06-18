use super::{armor::Armor, weapon::Weapon};
use crate::{
	kdl_ext::{AsKdl, FromKDL, NodeBuilder, NodeExt},
	system::dnd5e::{data::character::Character, BoxedCriteria, BoxedMutator},
	utility::MutatorGroup,
};
use std::path::Path;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Equipment {
	/// The criteria which must be met for this item to be equipped.
	pub criteria: Option<BoxedCriteria>,
	/// Passive mutators applied while this item is equipped.
	pub mutators: Vec<BoxedMutator>,
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
		for mutator in &self.mutators {
			mutator.set_data_path(path_to_item);
		}
		if let Some(armor) = &self.armor {
			armor.set_data_path(path_to_item);
		}
	}

	fn apply_mutators(&self, stats: &mut Character, path_to_item: &Path) {
		for modifier in &self.mutators {
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
			mutators,
			armor,
			shield,
			weapon,
			attunement,
		})
	}
}

impl AsKdl for Equipment {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		if let Some(armor) = &self.armor {
			node.push_child_t("armor", armor);
		}
		if let Some(shield) = &self.shield {
			node.push_child(
				NodeBuilder::default()
					.with_entry(("bonus", *shield as i64))
					.build("shield"),
			);
		}
		if let Some(weapon) = &self.weapon {
			node.push_child_t("weapon", weapon);
		}
		if let Some(_attunement) = &self.attunement {
			// TODO: Attunement node.push_child_t("attunement", attunement);
		}

		if let Some(criteria) = &self.criteria {
			node.push_child_t("criteria", criteria);
		}
		for mutator in &self.mutators {
			node.push_child_t("mutator", mutator);
		}

		node
	}
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Attunement {
	pub modifiers: Vec<BoxedMutator>,
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::{test_utils::*, NodeContext},
			system::{
				core::NodeRegistry,
				dnd5e::{
					data::{
						item::{armor, weapon},
						roll::{Die, Modifier, Roll},
						ArmorClassFormula, DamageType, Skill,
					},
					mutator::{AddModifier, ModifierKind},
				},
			},
			utility::Selector,
		};

		static NODE_NAME: &str = "equipment";

		fn node_ctx() -> NodeContext {
			NodeContext::registry(NodeRegistry::default_with_mut::<AddModifier>())
		}

		#[test]
		fn armor() -> anyhow::Result<()> {
			let doc = "
				|equipment {
				|    armor \"Heavy\" {
				|        formula base=18
				|        min-strength 15
				|    }
				|    mutator \"add_modifier\" \"Disadvantage\" (Skill)\"Specific\" \"Stealth\"
				|}
			";
			let data = Equipment {
				criteria: None,
				mutators: vec![AddModifier {
					modifier: Modifier::Disadvantage,
					context: None,
					kind: ModifierKind::Skill(Selector::Specific(Skill::Stealth)),
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
				shield: None,
				weapon: None,
				attunement: None,
			};
			assert_eq_fromkdl!(Equipment, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn weapon() -> anyhow::Result<()> {
			let doc = "
				|equipment {
				|    weapon \"Martial\" class=\"Maul\" {
				|        damage \"Bludgeoning\" roll=\"2d6\"
				|        property \"Heavy\"
				|        property \"TwoHanded\"
				|    }
				|}
			";
			let data = Equipment {
				criteria: None,
				mutators: vec![],
				armor: None,
				shield: None,
				weapon: Some(Weapon {
					kind: weapon::Kind::Martial,
					classification: "Maul".into(),
					damage: Some(weapon::WeaponDamage {
						roll: Some(Roll::from((2, Die::D6))),
						bonus: 0,
						damage_type: DamageType::Bludgeoning,
					}),
					properties: vec![weapon::Property::Heavy, weapon::Property::TwoHanded],
					range: None,
				}),
				attunement: None,
			};
			assert_eq_fromkdl!(Equipment, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn shield() -> anyhow::Result<()> {
			let doc = "
				|equipment {
				|    shield bonus=2
				|}
			";
			let data = Equipment {
				criteria: None,
				mutators: vec![],
				armor: None,
				shield: Some(2),
				weapon: None,
				attunement: None,
			};
			assert_eq_fromkdl!(Equipment, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
