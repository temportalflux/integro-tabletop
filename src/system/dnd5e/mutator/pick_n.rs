use crate::kdl_ext::NodeContext;
use crate::system::mutator::ReferencePath;
use crate::{
	system::dnd5e::{
		data::{character::Character, description},
		BoxedMutator,
	},
	system::Mutator,
	utility::selector,
};
use itertools::Itertools;
use kdlize::OmitIfEmpty;
use kdlize::{ext::DocumentExt, AsKdl, FromKdl, NodeBuilder};
use std::collections::{BTreeSet, HashMap, HashSet};

/// Allows the user to select some number of options where each option can apply a different group of mutators.
#[derive(Clone, PartialEq, Debug)]
pub struct PickN {
	name: String,
	id: String,
	options: HashMap<String, PickOption>,
	selector: selector::Value<Character, String>,
}

#[derive(Clone, PartialEq, Debug)]
struct PickOption {
	description: Option<description::Section>,
	mutators: Vec<BoxedMutator>,
}

crate::impl_trait_eq!(PickN);
kdlize::impl_kdl_node!(PickN, "pick");

impl PickN {
	fn id(&self) -> Option<std::borrow::Cow<'_, String>> {
		let selector::Value::Options(selector::ValueOptions { id, .. }) = &self.selector else {
			return None;
		};
		id.get_id()
	}

	fn max_selections(&self) -> usize {
		let selector::Value::Options(selector::ValueOptions { amount, .. }) = &self.selector else {
			return 0;
		};
		let crate::utility::Value::Fixed(amt) = amount else {
			return 0;
		};
		*amt as usize
	}

	fn option_order(&self) -> Option<&BTreeSet<String>> {
		let selector::Value::Options(selector::ValueOptions { options, .. }) = &self.selector else {
			return None;
		};
		Some(options)
	}

	fn get_selections_in<'this, 'c>(&'this self, state: Option<&'c Character>) -> HashSet<&'c String> {
		let Some((state, data_path)) = state.zip(self.selector.get_data_path()) else {
			return HashSet::default();
		};
		let Some(data) = state.get_selections_at(&data_path) else {
			return HashSet::default();
		};
		data.iter().collect::<HashSet<_>>()
	}
}

impl Mutator for PickN {
	type Target = Character;

	fn description(&self, state: Option<&Character>) -> description::Section {
		let selections = self.get_selections_in(state);

		let mut children = Vec::new();
		for key in self.option_order().unwrap() {
			// only show this option if: there are no options selected OR this option is selected
			if !selections.is_empty() && !selections.contains(key) {
				continue;
			};

			let Some(option) = self.options.get(key) else {
				continue;
			};
			let mut content = String::new().into();
			let mut option_children = Vec::new();
			if let Some(description::Section {
				title: _,
				content: option_content,
				format_args: _,
				children,
			}) = &option.description
			{
				content = option_content.clone();
				option_children.extend(children.iter().cloned());
			}
			for mutator in &option.mutators {
				let mut section = mutator.description(state);
				// if no option is selected, dont show the fields to select value for any particular option
				if selections.is_empty() {
					section.remove_selector_children();
				}
				option_children.push(section);
			}
			children.push(description::Section {
				title: Some(key.clone()),
				content,
				children: option_children,
				..Default::default()
			});
		}

		let selectors = selector::DataList::default().with_value("Selected Option", &self.selector, state);
		children.insert(0, selectors.into());

		description::Section {
			title: Some(self.name.clone()),
			content: format!(
				"Select {} of the following {} options.",
				self.max_selections(),
				self.options.len()
			)
			.into(),
			children,
			..Default::default()
		}
	}

	fn set_data_path(&self, parent: &ReferencePath) {
		self.selector.set_data_path(parent);
		for (name, option) in &self.options {
			let path_to_option = parent.join(&self.id, None).join(name, None);
			for mutator in &option.mutators {
				mutator.set_data_path(&path_to_option);
			}
		}
	}

	fn on_insert(&self, stats: &mut Self::Target, parent: &ReferencePath) {
		let Some(data_path) = self.selector.get_data_path() else {
			return;
		};
		let selected_options = {
			let Some(selections) = stats.get_selections_at(&data_path) else {
				return;
			};
			selections
				.iter()
				.filter_map(|key| self.options.get(key))
				.collect::<Vec<_>>()
		};
		for option in selected_options {
			for mutator in &option.mutators {
				stats.apply(mutator, parent);
			}
		}
	}
}

impl FromKdl<NodeContext> for PickN {
	type Error = anyhow::Error;
	fn from_kdl<'doc>(node: &mut crate::kdl_ext::NodeReader<'doc>) -> anyhow::Result<Self> {
		let max_selections = node.next_i64_req()? as usize;
		let name = node.get_str_req("name")?.to_owned();

		let id = match node.get_str_opt("id")? {
			Some(id) => id.to_owned(),
			None => name.clone(),
		};

		let mut cannot_match = Vec::new();
		for path_str in node.query_str_all("scope() > cannot_match", 0)? {
			cannot_match.push(path_str.into());
		}

		let mut options = HashMap::new();
		for node in &mut node.query_all("scope() > option")? {
			let name = node.next_str_req()?.to_owned();
			let description = node.query_opt_t::<description::Section>("scope() > description")?;
			let mutators = node.query_all_t("scope() > mutator")?;
			options.insert(name, PickOption { description, mutators });
		}

		let selector = selector::Value::Options(selector::ValueOptions {
			id: selector::IdPath::from(Some(id.clone())),
			amount: crate::utility::Value::Fixed(max_selections as i32),
			options: options.keys().cloned().sorted().collect(),
			cannot_match,
			is_applicable: None,
		});

		Ok(Self {
			name,
			id,
			options,
			selector,
		})
	}
}

impl AsKdl for PickN {
	fn as_kdl(&self) -> NodeBuilder {
		let mut node = NodeBuilder::default();

		node.push_entry(self.max_selections() as i64);

		node.push_entry(("name", self.name.clone()));
		if let Some(id) = self.id() {
			if *id != self.name {
				node.push_entry(("id", id.into_owned()));
			}
		}

		if let selector::Value::Options(selector::ValueOptions { cannot_match, .. }) = &self.selector {
			for id_path in cannot_match {
				let Some(id_str) = id_path.get_id() else {
					continue;
				};
				node.push_child_entry("cannot_match", id_str.into_owned());
			}
		}

		for name in self.option_order().unwrap() {
			let Some(option) = self.options.get(name) else {
				continue;
			};
			let mut node_option = NodeBuilder::default();
			node_option.push_entry(name.clone());
			node_option.push_child_t(("description", &option.description, OmitIfEmpty));
			node_option.push_children_t(("mutator", option.mutators.iter()));
			node.push_child(node_option.build("option"));
		}

		node
	}
}

#[cfg(test)]
mod test {
	use super::*;

	mod kdl {
		use super::*;
		use crate::{
			kdl_ext::test_utils::*,
			system::{
				dnd5e::{
					data::bounded::BoundValue,
					mutator::{test::test_utils, Speed},
				},
				generics,
			},
		};

		test_utils!(PickN, node_reg());

		fn node_reg() -> generics::Registry {
			let mut node_reg = generics::Registry::default();
			node_reg.register_mutator::<PickN>();
			node_reg.register_mutator::<Speed>();
			node_reg
		}

		fn options() -> HashMap<String, PickOption> {
			[
				(
					"Climbing".into(),
					PickOption {
						description: None,
						mutators: vec![Speed {
							name: "Climbing".into(),
							argument: BoundValue::Base(15),
						}
						.into()],
					},
				),
				(
					"Swimming".into(),
					PickOption {
						description: Some(description::Section {
							content: description::SectionContent::Body("You have a swimming speed of 15".into()),
							..Default::default()
						}),
						mutators: vec![Speed {
							name: "Swimming".into(),
							argument: BoundValue::Base(15),
						}
						.into()],
					},
				),
			]
			.into()
		}

		#[test]
		fn basic() -> anyhow::Result<()> {
			let doc = "
				|mutator \"pick\" 1 name=\"Default Speed\" {
				|    option \"Climbing\" {
				|        mutator \"speed\" \"Climbing\" (Base)15
				|    }
				|    option \"Swimming\" {
				|        description \"You have a swimming speed of 15\"
				|        mutator \"speed\" \"Swimming\" (Base)15
				|    }
				|}
			";
			let data = PickN {
				name: "Default Speed".into(),
				id: "Default Speed".into(),
				options: options(),
				selector: selector::Value::Options(selector::ValueOptions {
					id: "Default Speed".into(),
					options: ["Climbing".into(), "Swimming".into()].into(),
					..Default::default()
				}),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn with_id() -> anyhow::Result<()> {
			let doc = "
				|mutator \"pick\" 1 name=\"Default Speed\" id=\"speedA\" {
				|    option \"Climbing\" {
				|        mutator \"speed\" \"Climbing\" (Base)15
				|    }
				|    option \"Swimming\" {
				|        description \"You have a swimming speed of 15\"
				|        mutator \"speed\" \"Swimming\" (Base)15
				|    }
				|}
			";
			let data = PickN {
				name: "Default Speed".into(),
				id: "speedA".into(),
				options: options(),
				selector: selector::Value::Options(selector::ValueOptions {
					id: "speedA".into(),
					options: ["Climbing".into(), "Swimming".into()].into(),
					..Default::default()
				}),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}

		#[test]
		fn with_cannot_match() -> anyhow::Result<()> {
			let doc = "
				|mutator \"pick\" 1 name=\"Default Speed\" id=\"speedA\" {
				|    cannot_match \"/RootFeature/some_value\"
				|    option \"Climbing\" {
				|        mutator \"speed\" \"Climbing\" (Base)15
				|    }
				|    option \"Swimming\" {
				|        description \"You have a swimming speed of 15\"
				|        mutator \"speed\" \"Swimming\" (Base)15
				|    }
				|}
			";
			let data = PickN {
				name: "Default Speed".into(),
				id: "speedA".into(),
				options: options(),
				selector: selector::Value::Options(selector::ValueOptions {
					id: "speedA".into(),
					options: ["Climbing".into(), "Swimming".into()].into(),
					cannot_match: ["/RootFeature/some_value".into()].into(),
					..Default::default()
				}),
			};
			assert_eq_askdl!(&data, doc);
			assert_eq_fromkdl!(Target, doc, data.into());
			Ok(())
		}
	}
}
