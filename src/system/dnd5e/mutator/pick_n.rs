use itertools::Itertools;

use crate::{
	kdl_ext::{DocumentExt, FromKDL, NodeExt},
	system::dnd5e::{
		data::{character::Character, description, Feature},
		BoxedMutator,
	},
	utility::{IdPath, Mutator, MutatorGroup, Selector, SelectorMetaVec},
};
use std::collections::HashMap;

/// Allows the user to select some number of options where each option can apply a different group of mutators.
#[derive(Clone, PartialEq, Debug)]
pub struct PickN {
	name: String,
	options: HashMap<String, PickOption>,
	selector: Selector<String>,
}

#[derive(Clone, PartialEq, Debug)]
struct PickOption {
	description: Option<description::Section>,
	mutators: Vec<BoxedMutator>,
	features: Vec<Feature>,
}

crate::impl_trait_eq!(PickN);
crate::impl_kdl_node!(PickN, "pick");

impl PickN {
	fn max_selections(&self) -> usize {
		let Selector::AnyOf { amount, .. } = &self.selector else { return 0; };
		*amount
	}

	fn option_order(&self) -> Option<&Vec<String>> {
		let Selector::AnyOf { options, .. } = &self.selector else { return None; };
		Some(options)
	}
}

impl Mutator for PickN {
	type Target = Character;

	fn description(&self) -> description::Section {
		let selectors = SelectorMetaVec::default().with_str("Selected Option", &self.selector);
		let mut children = Vec::new();
		for key in self.option_order().unwrap() {
			let Some(option) = self.options.get(key) else { continue; };
			let mut content = String::new();
			let mut option_children = Vec::new();
			if let Some(description::Section {
				title: _,
				content: option_content,
				selectors: _,
				kind,
			}) = &option.description {
				content = option_content.clone();
				match kind {
					None => {}
					Some(description::SectionKind::HasChildren(children)) => {
						option_children.extend(children.iter().cloned());
					}
				}
			}
			for mutator in &option.mutators {
				option_children.push(mutator.description());
			}
			for feature in &option.features {
				option_children.extend(feature.description.long.iter().cloned());
			}
			children.push(description::Section {
				title: Some(key.clone()),
				content,
				kind: Some(description::SectionKind::HasChildren(option_children)),
				..Default::default()
			});
		}
		description::Section {
			title: Some(self.name.clone()),
			content: format!("Select {} of the following {} options.", self.max_selections(), self.options.len()),
			selectors,
			kind: Some(description::SectionKind::HasChildren(children)),
		}
	}

	fn set_data_path(&self, parent: &std::path::Path) {
		self.selector.set_data_path(parent);
		for (name, option) in &self.options {
			let path_to_option = parent.join(name);
			for mutator in &option.mutators {
				mutator.set_data_path(&path_to_option);
			}
			for feature in &option.features {
				feature.set_data_path(&path_to_option);
			}
		}
	}

	fn apply(&self, stats: &mut Self::Target, parent: &std::path::Path) {
		let Some(data_path) = self.selector.get_data_path() else { return; };
		let selected_options = {
			let Some(selections) = stats.get_selections_at(&data_path) else { return; };
			selections
				.iter()
				.filter_map(|key| self.options.get(key))
				.collect::<Vec<_>>()
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

			let description = match node.query_opt("scope() > description")? {
				None => None,
				Some(node) => {
					Some(description::Section::from_kdl(node, &mut ctx.next_node())?)
				}
			};

			let mut mutators = Vec::new();
			for entry_node in node.query_all("scope() > mutator")? {
				mutators.push(ctx.parse_mutator(entry_node)?);
			}

			let mut features = Vec::new();
			for entry_node in node.query_all("scope() > feature")? {
				features.push(Feature::from_kdl(entry_node, &mut ctx.next_node())?.into());
			}

			options.insert(name, PickOption { description, mutators, features });
		}

		let selector = Selector::AnyOf {
			id,
			cannot_match,
			amount: max_selections,
			options: options.keys().cloned().sorted().collect(),
		};

		Ok(Self {
			name,
			options,
			selector,
		})
	}
}
