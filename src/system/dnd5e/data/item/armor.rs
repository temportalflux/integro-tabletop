use crate::{
	kdl_ext::{DocumentExt, NodeExt, ValueIdx},
	system::dnd5e::{
		data::{character::Character, mutator::ArmorStrengthRequirement, ArmorClassFormula},
		FromKDL,
	},
	utility::MutatorGroup,
	GeneralError,
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
		value_idx: &mut ValueIdx,
		node_reg: &crate::system::core::NodeRegistry,
	) -> anyhow::Result<Self> {
		let kind = Kind::from_str(node.get_str_req(value_idx.next())?)?;
		let formula = node
			.query("scope() > formula")?
			.ok_or(GeneralError(format!(
				"Node {node:?} must have a child node named \"formula\"."
			)))?;
		let formula = ArmorClassFormula::from_kdl(formula, &mut ValueIdx::default(), node_reg)?;
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
	type Err = crate::GeneralError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"light" => Ok(Self::Light),
			"medium" => Ok(Self::Medium),
			"heavy" => Ok(Self::Heavy),
			_ => Err(crate::GeneralError(format!(
				"{s:?} is not a valid armor kind: {:?}.",
				enumset::EnumSet::<Kind>::all()
					.into_iter()
					.collect::<Vec<_>>(),
			))),
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
