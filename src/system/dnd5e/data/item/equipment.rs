use super::{armor::Armor, weapon::Weapon};
use crate::kdl_ext::NodeContext;
use crate::system::dnd5e::data::roll::Roll;
use crate::system::dnd5e::data::Rest;
use crate::{
	system::dnd5e::{data::character::Character, BoxedCriteria, BoxedMutator},
	utility::MutatorGroup,
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};
use std::{collections::HashMap, path::Path};

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
	pub charges: Option<Charges>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Charges {
	pub capacity: usize,
	pub reset: Option<ChargesReset>,
}
#[derive(Clone, PartialEq, Debug)]
pub struct ChargesReset {
	pub roll: Roll,
	pub base: usize,
	pub rest: Rest,
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
				.push_bonus(*shield, None, path_to_item.to_owned());
		}
	}
}

impl Equipment {
	pub fn to_metadata(self) -> serde_json::Value {
		let mut contents: HashMap<&'static str, serde_json::Value> = [].into();
		if let Some(weapon) = self.weapon {
			contents.insert("weapon", weapon.to_metadata());
		}
		serde_json::json!(contents)
	}

	/// Returs Ok if the item can currently be equipped, otherwise returns a user-displayable reason why it cannot be equipped.
	pub fn can_be_equipped(&self, state: &Character) -> Result<(), String> {
		match &self.criteria {
			Some(criteria) => state.evaluate(criteria),
			None => Ok(()),
		}
	}
}

impl FromKdl<NodeContext> for Equipment {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let criteria = node.query_opt_t("scope() > criteria")?;
		let mutators = node.query_all_t("scope() > mutator")?;

		let armor = node.query_opt_t::<Armor>("scope() > armor")?;
		let shield = match node.query_opt("scope() > shield")? {
			None => None,
			Some(node) => Some(node.get_i64_req("bonus")? as i32),
		};
		let weapon = node.query_opt_t::<Weapon>("scope() > weapon")?;
		let attunement = match node.query_opt("scope() > attunement")? {
			None => None,
			Some(mut _node) => {
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
			charges: None,
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
			utility::selector,
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
				..Default::default()
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
				shield: Some(2),
				..Default::default()
			};
			assert_eq_fromkdl!(Equipment, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
