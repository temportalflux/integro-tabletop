use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::dnd5e::data::{character::Character, description, roll::Roll},
	utility::Mutator,
};

#[derive(Clone, PartialEq, Debug)]
pub struct AddSize {
	height: Vec<FormulaComponent>,
	weight: Vec<FormulaComponent>,
}
#[derive(Clone, PartialEq, Debug)]
enum FormulaComponent {
	Base(u32),
	Bonus(Roll),
	WeightMultiplier(Roll),
}
impl ToString for FormulaComponent {
	fn to_string(&self) -> String {
		match self {
			Self::Base(v) => v.to_string(),
			Self::Bonus(roll) => roll.to_string(),
			Self::WeightMultiplier(roll) => format!("{} (x height bonus)", roll.to_string()),
		}
	}
}

crate::impl_trait_eq!(AddSize);
crate::impl_kdl_node!(AddSize, "add_size");

impl Mutator for AddSize {
	type Target = Character;

	fn description(&self, _state: Option<&Character>) -> description::Section {
		let mut content = Vec::new();
		if !self.height.is_empty() {
			let comps = self
				.height
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>();
			let desc = comps.join(" + ");
			content.push(format!("Your height increases by {desc} inches."));
		}
		if !self.weight.is_empty() {
			let comps = self
				.weight
				.iter()
				.map(ToString::to_string)
				.collect::<Vec<_>>();
			let desc = comps.join(" + ");
			content.push(format!("Your weight increases by {desc} lbs."));
		}
		description::Section {
			content: content.join(" "),
			..Default::default()
		}
	}

	fn apply(&self, stats: &mut Character, _parent: &std::path::Path) {
		let size_formula = &mut stats.derived_description_mut().size_formula;
		for comp in &self.height {
			match comp {
				FormulaComponent::Base(value) => size_formula.height.base += *value,
				FormulaComponent::Bonus(roll) => size_formula.height.bonus.push(*roll),
				FormulaComponent::WeightMultiplier(_) => {}
			}
		}
		for comp in &self.weight {
			match comp {
				FormulaComponent::Base(value) => size_formula.weight.base += *value,
				FormulaComponent::Bonus(roll) => size_formula.weight.bonus.push(*roll),
				FormulaComponent::WeightMultiplier(roll) => {
					size_formula.weight.multiplier.push(*roll)
				}
			}
		}
	}
}

impl FromKDL for AddSize {
	fn from_kdl(
		node: &kdl::KdlNode,
		_ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let height = match node.query_opt("scope() > height")? {
			None => Vec::new(),
			Some(node) => {
				let mut comps = Vec::new();
				if let Some(base) = node.get_i64_opt("base")? {
					comps.push(FormulaComponent::Base(base as u32));
				}
				if let Some(kdl_value) = node.get("bonus") {
					comps.push(FormulaComponent::Bonus(Roll::from_kdl_value(kdl_value)?));
				}
				comps
			}
		};
		let weight = match node.query_opt("scope() > weight")? {
			None => Vec::new(),
			Some(node) => {
				let mut comps = Vec::new();
				if let Some(base) = node.get_i64_opt("base")? {
					comps.push(FormulaComponent::Base(base as u32));
				}
				if let Some(kdl_value) = node.get("bonus") {
					comps.push(FormulaComponent::Bonus(Roll::from_kdl_value(kdl_value)?));
				}
				if let Some(kdl_value) = node.get("multiplier") {
					comps.push(FormulaComponent::WeightMultiplier(Roll::from_kdl_value(
						kdl_value,
					)?));
				}
				comps
			}
		};
		Ok(Self { height, weight })
	}
}

// TODO: Test AddSize
