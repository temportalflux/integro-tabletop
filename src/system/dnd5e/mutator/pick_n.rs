use std::collections::HashMap;
use crate::{
	kdl_ext::{FromKDL, NodeExt, DocumentExt},
	system::dnd5e::{data::{character::Character, description, Feature}, BoxedMutator},
	utility::{Mutator, MutatorGroup, IdPath, Selector},
};

/// Allows the user to select some number of options where each option can apply a different group of mutators.
#[derive(Clone, PartialEq, Debug)]
pub struct PickN {
	name: String,
	options: HashMap<String, PickOption>,
	selector: Selector<String>,
}

#[derive(Clone, PartialEq, Debug)]
struct PickOption {
	mutators: Vec<BoxedMutator>,
	features: Vec<Feature>,
}

crate::impl_trait_eq!(PickN);
crate::impl_kdl_node!(PickN, "pick");

impl Mutator for PickN {
	type Target = Character;

	fn description(&self) -> description::Section {
		description::Section { title: Some(self.name.clone()), content: "".into(), ..Default::default() }
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.selector.set_data_path(parent);
		for (_name, option) in &self.options {
			for mutator in &option.mutators {
				mutator.set_data_path(parent);
			}
			for feature in &option.features {
				feature.set_data_path(parent);
			}
		}
	}

	fn apply(&self, stats: &mut Self::Target, parent: &std::path::Path) {
		let Some(data_path) = self.selector.get_data_path() else { return; };
		let selected_options = {
			let Some(selections) = stats.get_selections_at(&data_path) else { return; };
			selections.iter().filter_map(|key| self.options.get(key)).collect::<Vec<_>>()
		};
		for option in selected_options {
			for mutator in &option.mutators {
				stats.apply(mutator, parent);
			}
			for feature in &option.features {
				stats.add_feature(feature, parent);
			}
		}
	}
}

impl FromKDL for PickN {
	fn from_kdl(
		node: &kdl::KdlNode,
		ctx: &mut crate::kdl_ext::NodeContext,
	) -> anyhow::Result<Self> {
		let max_selections = node.get_i64_req(ctx.consume_idx())? as usize;
		let name = node.get_str_req("name")?.to_owned();
		
		let id = IdPath::from(node.get_str_opt("id")?);
		let cannot_match = node.query_str_all("scope() > cannot-match", 0)?;
		let cannot_match = cannot_match.into_iter().map(IdPath::from).collect();

		let mut options = HashMap::new();
		for node in node.query_all("scope() > option")? {
			let mut ctx = ctx.next_node();
			let name = node.get_str_req(ctx.consume_idx())?.to_owned();

			// TODO: Options can have a description

			let mut mutators = Vec::new();
			for entry_node in node.query_all("scope() > mutator")? {
				mutators.push(ctx.parse_mutator(entry_node)?);
			}
	
			let mut features = Vec::new();
			for entry_node in node.query_all("scope() > feature")? {
				features.push(Feature::from_kdl(entry_node, &mut ctx.next_node())?.into());
			}

			options.insert(name, PickOption { mutators, features });
		}

		let selector = Selector::AnyOf { id, cannot_match, amount: max_selections, options: options.keys().cloned().collect() };

		Ok(Self {
			name,
			options,
			selector,
		})
	}
}
