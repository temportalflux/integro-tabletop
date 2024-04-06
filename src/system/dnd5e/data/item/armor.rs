use crate::{
	kdl_ext::NodeContext,
	system::{
		dnd5e::{
			data::{character::Character, ArmorClassFormula},
			mutator::ArmorStrengthRequirement,
		},
		mutator::{self, ReferencePath},
	},
	utility::InvalidEnumStr,
};
use enumset::EnumSetType;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct Armor {
	pub kind: Kind,
	pub formula: ArmorClassFormula,
	/// The minimum expected strength score to use this armor.
	/// If provided, characters with a value less than this are hindered (reduced speed).
	pub min_strength_score: Option<u32>,
}

impl Armor {
	pub fn to_metadata(self) -> serde_json::Value {
		serde_json::json!({
			"kind": self.kind.to_string(),
		})
	}
}

impl FromKdl<NodeContext> for Armor {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let kind = node.next_str_req_t::<Kind>()?;
		let formula = node.query_req_t::<ArmorClassFormula>("scope() > formula")?;
		let min_strength_score = node.query_i64_opt("scope() > min-strength", 0)?.map(|v| v as u32);
		Ok(Self {
			kind,
			formula,
			min_strength_score,
		})
	}
}

impl AsKdl for Armor {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		node.entry(self.kind.to_string());
		node.child(("formula", &self.formula));
		node.child(("min-strength", &self.min_strength_score));
		node
	}
}

#[derive(Debug, PartialOrd, Ord, Hash, EnumSetType)]
pub enum Kind {
	Light,
	Medium,
	Heavy,
}
impl ToString for Kind {
	fn to_string(&self) -> String {
		format!("{self:?}")
	}
}
impl FromStr for Kind {
	type Err = InvalidEnumStr<Self>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"light" => Ok(Self::Light),
			"medium" => Ok(Self::Medium),
			"heavy" => Ok(Self::Heavy),
			_ => Err(InvalidEnumStr::from(s)),
		}
	}
}

impl mutator::Group for Armor {
	type Target = Character;

	fn set_data_path(&self, _path_to_item: &ReferencePath) {}

	fn apply_mutators(&self, stats: &mut Character, path_to_item: &ReferencePath) {
		stats.armor_class_mut().push_formula(self.formula.clone(), path_to_item);

		if let Some(min_strength_score) = &self.min_strength_score {
			let mutator = ArmorStrengthRequirement {
				score: *min_strength_score,
			};
			stats.apply(&mutator.into(), path_to_item);
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::dnd5e::data::{Ability, BoundedAbility},
		};

		static NODE_NAME: &str = "armor";

		#[test]
		fn light() -> anyhow::Result<()> {
			let doc = "
			|armor \"Light\" {
			|    formula base=11 {
			|        bonus (Ability)\"Dexterity\"
			|    }
			|}
		";
			let data = Armor {
				kind: Kind::Light,
				formula: ArmorClassFormula {
					base: 11,
					bonuses: vec![BoundedAbility {
						ability: Ability::Dexterity,
						min: None,
						max: None,
					}],
				},
				min_strength_score: None,
			};
			assert_eq_fromkdl!(Armor, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn medium() -> anyhow::Result<()> {
			let doc = "
			|armor \"Medium\" {
			|    formula base=13 {
			|        bonus (Ability)\"Dexterity\" max=2
			|    }
			|}
		";
			let data = Armor {
				kind: Kind::Medium,
				formula: ArmorClassFormula {
					base: 13,
					bonuses: vec![BoundedAbility {
						ability: Ability::Dexterity,
						min: None,
						max: Some(2),
					}],
				},
				min_strength_score: None,
			};
			assert_eq_fromkdl!(Armor, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}

		#[test]
		fn heavy() -> anyhow::Result<()> {
			let doc = "
			|armor \"Heavy\" {
			|    formula base=18
			|    min-strength 15
			|}
		";
			let data = Armor {
				kind: Kind::Heavy,
				formula: ArmorClassFormula {
					base: 18,
					bonuses: vec![],
				},
				min_strength_score: Some(15),
			};
			assert_eq_fromkdl!(Armor, doc, data);
			assert_eq_askdl!(&data, doc);
			Ok(())
		}
	}
}
