use crate::{
	kdl_ext::NodeContext,
	system::dnd5e::data::{item::equipment::Equipment, ArmorClassFormula},
};
use kdlize::{AsKdl, FromKdl, NodeBuilder};

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorExtension {
	pub formula: Option<ArmorFormulaExtension>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ArmorFormulaExtension {
	pub base_bonus: Option<i32>,
}

impl ArmorExtension {
	pub fn apply_to(&self, equipment: &mut Equipment) -> anyhow::Result<()> {
		let Some(armor) = &mut equipment.armor else {
			return Ok(());
		};
		if let Some(formula_ext) = &self.formula {
			formula_ext.apply_to(&mut armor.formula)?;
		}
		Ok(())
	}
}

impl ArmorFormulaExtension {
	pub fn apply_to(&self, formula: &mut ArmorClassFormula) -> anyhow::Result<()> {
		if let Some(base_bonus) = &self.base_bonus {
			formula.base = formula.base.saturating_add_signed(*base_bonus);
		}
		Ok(())
	}
}

impl FromKdl<NodeContext> for ArmorExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let formula = node.query_opt_t("scope() > formula")?;
		Ok(Self { formula })
	}
}

impl AsKdl for ArmorExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(formula) = &self.formula {
			node.push_child_t(("formula", formula));
		}
		node
	}
}

impl FromKdl<NodeContext> for ArmorFormulaExtension {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let base_bonus = node.get_i64_opt("base_bonus")?.map(|num| num as i32);
		Ok(Self { base_bonus })
	}
}

impl AsKdl for ArmorFormulaExtension {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();
		if let Some(base_bonus) = &self.base_bonus {
			node.push_entry(("base_bonus", *base_bonus as i64));
		}
		node
	}
}
