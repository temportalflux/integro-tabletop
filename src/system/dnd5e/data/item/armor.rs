use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::dnd5e::data::{
		character::Character, mutator::ArmorStrengthRequirement, ArmorClassFormula,
	},
	utility::{InvalidEnumStr, MutatorGroup},
};
use enumset::EnumSetType;
use std::{path::Path, str::FromStr};

#[derive(Clone, PartialEq, Debug)]
pub struct Armor {
	pub kind: Kind,
	pub formula: ArmorClassFormula,
	/// The minimum expected strength score to use this armor.
	/// If provided, characters with a value less than this are hindered (reduced speed).
	pub min_strength_score: Option<u32>,
}

impl FromKDL for Armor {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let kind = Kind::from_str(node.get_str_req(ctx.consume_idx())?)?;
		let formula = node.query_req("scope() > formula")?;
		let formula = ArmorClassFormula::from_kdl(formula, &mut ctx.next_node())?;
		let min_strength_score = node
			.query_i64_opt("scope() > min-strength", 0)?
			.map(|v| v as u32);
		Ok(Self {
			kind,
			formula,
			min_strength_score,
		})
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

impl MutatorGroup for Armor {
	type Target = Character;

	fn set_data_path(&self, _path_to_item: &std::path::Path) {}

	fn apply_mutators(&self, stats: &mut Character, path_to_item: &Path) {
		stats
			.armor_class_mut()
			.push_formula(self.formula.clone(), path_to_item.to_owned());

		if let Some(min_strength_score) = &self.min_strength_score {
			let mutator = ArmorStrengthRequirement {
				score: *min_strength_score,
			};
			stats.apply(&mutator.into(), path_to_item);
		}
	}
}

// TODO: Test Armor
